mod delays;
mod motion;
mod motion_util;
mod stats;
mod time;
use crate::options::GCodeFlavor;
use delays::command_delay;
#[cfg(test)]
use motion::planned_times;
use motion::{MotionBlock, MotionKind, MotionState, RollingPlanner};
use motion_util::word;
pub(super) use stats::{PRINT_TIME_SEC_PLACEHOLDER, USED_FILAMENT_LENGTH_PLACEHOLDER};
use std::collections::VecDeque;
use time::{duration, minutes};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct ProcessorLimits {
    pub(super) print_acceleration: f64,
    pub(super) retract_acceleration: f64,
    pub(super) travel_acceleration: f64,
    pub(super) gcode_flavor: GCodeFlavor,
    pub(super) bbl_printer: bool,
    pub(super) junction_deviation: f64,
}

pub(super) fn process(
    mut output: Vec<u8>,
    emit_progress: bool,
    machine_load_filament_time: f64,
    used_filament: f64,
    limits: ProcessorLimits,
) -> Vec<u8> {
    let text = String::from_utf8(std::mem::take(&mut output)).expect("generated G-code is UTF-8");
    // Debug instrument: estimate times over an external g-code file instead
    // of the freshly sliced output (ARES_ESTIMATE_ONLY=path).
    let text = match std::env::var("ARES_ESTIMATE_ONLY") {
        Ok(path) if !path.is_empty() => {
            std::fs::read_to_string(path).expect("ARES_ESTIMATE_ONLY file readable")
        }
        _ => text,
    };
    let lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    let estimate = Estimate::from_lines(&lines, machine_load_filament_time, limits);
    let mut result = String::with_capacity(text.len() + text.len() / 100);
    let mut last_progress = None;
    let first_marker = lines
        .iter()
        .position(|line| line == "M73 P0 R0")
        .unwrap_or(0);
    let prepare_time = estimate.prepare;
    let model_time = (estimate.total - prepare_time).max(0.0);
    let print_time_sec = format!("{:.2}", estimate.total);
    let used_filament_length = format!("{:.2}", used_filament / 1_000.0);

    for (index, line) in lines.iter().enumerate() {
        let expanded = stats::expand(line, &print_time_sec, &used_filament_length);
        let line = expanded.as_ref();
        if line == "M73 P0 R0" {
            if emit_progress {
                let remaining = minutes(estimate.total);
                last_progress = Some((0, remaining));
                result.push_str(&format!("M73 P0 R{remaining}\n"));
            }
            continue;
        }
        if !emit_progress && line == "M73 P100 R0" {
            continue;
        }
        if line.starts_with("; model printing time:")
            || line == "; estimated printing time (normal mode) = 0s"
        {
            if limits.bbl_printer {
                result.push_str(&format!(
                    "; model printing time: {}; total estimated time: {}\n",
                    duration(model_time),
                    duration(estimate.total),
                ));
            } else {
                result.push_str(&format!(
                    "; estimated printing time (normal mode) = {}\n",
                    duration(estimate.total),
                ));
            }
            continue;
        }
        if line.starts_with("; estimated first layer printing time") {
            result.push_str(&format!(
                "; estimated first layer printing time (normal mode) = {}\n",
                duration(prepare_time),
            ));
            continue;
        }
        result.push_str(line);
        result.push('\n');
        if index < first_marker || !emit_progress || !is_progress_motion(line) {
            continue;
        }
        let Some(elapsed) = estimate.elapsed_at(index + 1).map(|elapsed| elapsed as f32) else {
            continue;
        };
        let percent = if estimate.total > 0.0 {
            (f64::from(100.0_f32 * elapsed) / estimate.total).clamp(0.0, 99.0) as u64
        } else {
            0
        };
        let remaining = minutes(estimate.total - f64::from(elapsed));
        if last_progress != Some((percent, remaining)) {
            last_progress = Some((percent, remaining));
            result.push_str(&format!("M73 P{percent} R{remaining}\n"));
        }
    }
    output.clear();
    output.extend_from_slice(result.as_bytes());
    output
}

fn is_progress_motion(line: &str) -> bool {
    matches!(
        line.split(';')
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .next(),
        Some("G0" | "G1" | "G2" | "G3")
    )
}

struct Estimate {
    total: f64,
    prepare: f64,
    elapsed: Vec<Option<f64>>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum DelayTarget {
    Any,
    ToolChange,
}

#[derive(Clone, Copy)]
struct PendingDelay {
    target: DelayTarget,
    seconds: f32,
}

struct FlushEvent {
    block_count: usize,
    delay: Option<PendingDelay>,
}

impl Estimate {
    fn from_lines(
        lines: &[String],
        machine_load_filament_time: f64,
        limits: ProcessorLimits,
    ) -> Self {
        let mut blocks = Vec::new();
        let travel_limit = if limits.gcode_flavor.supports_separate_travel_acceleration() {
            limits.travel_acceleration
        } else {
            0.0
        };
        let mut state = MotionState::with_acceleration_limits(
            limits.print_acceleration,
            limits.retract_acceleration,
            travel_limit,
        );
        let mut prepare_stages = Vec::new();
        state.gcode_flavor = limits.gcode_flavor;
        state.junction_deviation = limits.junction_deviation;
        let mut events = Vec::new();
        let mut prepare_stage = false;
        let mut saw_motion_command = false;
        let mut measure_g29_time = false;
        let mut active_tool = None;
        let mut g1_line_id = 0;
        let mut block_line_ids = Vec::new();
        let mut arc_segment_counts = vec![0; lines.len()];
        for (index, line) in lines.iter().enumerate() {
            match line.trim() {
                "; WIPE_START" | ";WIPE_START" => state.set_wiping(true),
                "; WIPE_END" | ";WIPE_END" => state.set_wiping(false),
                "; FEATURE: Custom" | ";TYPE:Custom" => prepare_stage = !saw_motion_command,
                line if line.starts_with("; FEATURE:") || line.starts_with(";TYPE:") => {
                    prepare_stage = false
                }
                _ => {}
            }
            let code = line.split(';').next().unwrap_or_default().trim();
            if code.starts_with("M622") && word(code, 'J').unwrap_or(0.0).round() == 1.0 {
                measure_g29_time = true;
            } else if code.starts_with("M623") {
                measure_g29_time = false;
            }
            let is_g29 = code.starts_with("G29") && !code.starts_with("G29.");
            // Orca counts the hardcoded 260s G29 bedding delay on every
            // machine; the M622 J1 gate only applies to BBL printers
            // (`GCodeProcessor.cpp:4859-4869`).
            let bbl_gating_blocks_delay = limits.bbl_printer && !measure_g29_time;
            let delay = match is_g29 {
                true if bbl_gating_blocks_delay => None,
                _ => command_delay(code),
            };
            let measured_g29 = is_g29 && !bbl_gating_blocks_delay;
            if synchronizes_planner(code, measured_g29) {
                events.push(FlushEvent {
                    block_count: blocks.len(),
                    delay: delay.map(|seconds| PendingDelay {
                        target: DelayTarget::Any,
                        seconds: seconds as f32,
                    }),
                });
            }
            if selects_initial_tool(code, &mut active_tool) {
                blocks.push(MotionBlock {
                    distance: 0.0,
                    speed: 0.0,
                    acceleration: 0.0,
                    centripetal_acceleration: 0.0,
                    jerk: [0.0; 4],
                    direction: [0.0; 4],
                    // The tool-change block lands no `g1_times_cache` entry
                    // (verified against the `ORCA_DUMP_TIMES` dump: the
                    // initial `T0`'s load delay reaches the machine total via
                    // blocks that the M73 lookup never sees), so M73 emission
                    // never attaches to the tool-change line.
                    e_only: true,
                    kind: MotionKind::ToolChange,
                });
                block_line_ids.push(g1_line_id);
                prepare_stages.push(prepare_stage);
                events.push(FlushEvent {
                    block_count: blocks.len(),
                    delay: Some(PendingDelay {
                        target: DelayTarget::ToolChange,
                        seconds: machine_load_filament_time as f32,
                    }),
                });
            }
            let command = code.split_whitespace().next().unwrap_or_default();
            if let Ok(path) = std::env::var("ARES_DUMP_LMAP2") {
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    let _ = writeln!(file, "P1 idx={index} id={g1_line_id} line={code}");
                }
            }
            let arc_internal = matches!(command, "G2" | "G3")
                .then(|| arc_internal_g1_lines(code, command, &state));
            let mut motion_blocks = state.motions(code);
            // The G1 immediately preceding ;WIPE_START is the pre-wipe
            // inward move — upstream keeps its time but drops its cache
            // entry.
            let inward_next = lines
                .get(index + 1)
                .is_some_and(|next| next.trim_start().starts_with(";WIPE_START"));
            if inward_next {
                for block in &mut motion_blocks {
                    if !block.e_only {
                        block.kind = MotionKind::InwardMove;
                    }
                }
            }
            match command {
                "G0" | "G1" | "G28" => {
                    g1_line_id += 1;
                    let count = motion_blocks.len();
                    blocks.extend(motion_blocks);
                    block_line_ids.extend(std::iter::repeat_n(g1_line_id, count));
                    prepare_stages.extend(std::iter::repeat_n(prepare_stage, count));
                }
                "G2" | "G3" => {
                    let count = motion_blocks.len();
                    arc_segment_counts[index] = arc_internal.unwrap_or(0);
                    blocks.extend(motion_blocks);
                    // The arc line plus its internal discretized G1s consume
                    // `1 + internal` g1 line ids; all planner segments carry
                    // the arc line's own id.
                    g1_line_id += 1;
                    block_line_ids.extend(std::iter::repeat_n(g1_line_id, count));
                    prepare_stages.extend(std::iter::repeat_n(prepare_stage, count));
                    g1_line_id += arc_internal.unwrap_or(0);
                }
                _ => debug_assert!(motion_blocks.is_empty()),
            }
            saw_motion_command |= is_progress_motion(code);
        }
        let (times, trailing_delay) = scheduled_times(&blocks, &events);
        debug_assert_eq!(block_line_ids.len(), times.len());
        let block_line_ids_original = block_line_ids.clone();
        let mut cumulative = 0.0;
        if let Ok(path) = std::env::var("ARES_DUMP_EBLOCKS") {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let mut running = 0.0;
                for ((id, time), block) in block_line_ids_original
                    .iter()
                    .zip(times.iter())
                    .zip(blocks.iter())
                {
                    running += *time;
                    if block.e_only {
                        let _ = writeln!(
                            file,
                            "EBLOCK id={id} t={:.6} dist={:.6} v={:.3} accel={:.3} jerk=({:.1},{:.1},{:.1},{:.1})",
                            running,
                            block.distance,
                            block.speed,
                            block.acceleration,
                            block.jerk[0],
                            block.jerk[1],
                            block.jerk[2],
                            block.jerk[3]
                        );
                    }
                }
            }
        }
        let cache = block_line_ids
            .into_iter()
            .zip(&times)
            .zip(blocks.iter().map(|block| block.e_only))
            .zip(
                blocks
                    .iter()
                    .map(|block| block.kind == MotionKind::InwardMove),
            )
            .filter_map(|(((id, time), e_only), inward)| {
                cumulative += time;
                // E-only moves time into the total but get no g1_times_cache
                // entry, so M73 emission skips retract/unretract lines
                // (mirrors the GT dump behavior). The pre-wipe inward move
                // also gets no cache entry upstream (verified via the
                // ORCA_DUMP_LINES/ORCA_DUMP_MTYPE bilateral dumps: all 62
                // wipe-tail travels vanish from the g1_times_cache while
                // their time still counts).
                (!e_only && !inward).then_some((id, cumulative))
            })
            .collect::<Vec<_>>();
        let mut elapsed = vec![None; lines.len() + 1];
        let mut cache_index = 0;
        let mut exported_g1_lines = 0;
        for (index, line) in lines.iter().enumerate() {
            let command = line
                .split(';')
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .next()
                .unwrap_or_default();
            let lookup_id = match command {
                "G0" | "G1" => {
                    let id = exported_g1_lines;
                    exported_g1_lines += 1;
                    Some(id)
                }
                "G2" | "G3" => {
                    let id = exported_g1_lines;
                    exported_g1_lines += 1 + arc_segment_counts[index];
                    Some(id)
                }
                "G28" => {
                    exported_g1_lines += 1;
                    None
                }
                _ => None,
            };
            let Some(lookup_id) = lookup_id else {
                continue;
            };
            if let Ok(path) = std::env::var("ARES_DUMP_LMAP") {
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    let _ = writeln!(
                        file,
                        "LM idx={index} id={lookup_id} line={}",
                        line.split(';').next().unwrap_or_default().trim()
                    );
                }
            }
            while cache_index < cache.len() && cache[cache_index].0 < lookup_id {
                cache_index += 1;
            }
            if cache
                .get(cache_index)
                .is_some_and(|(id, _)| *id == lookup_id)
            {
                elapsed[index + 1] = Some(cache[cache_index].1);
            }
        }
        let prepare = times
            .iter()
            .zip(prepare_stages)
            .filter_map(|(time, is_prepare)| is_prepare.then_some(time))
            .sum();
        if let Ok(path) = std::env::var("ARES_DUMP_TIMES") {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(
                    file,
                    "ESTIMATE total {} blocks {}",
                    cumulative + trailing_delay,
                    blocks.len()
                );
                for ((id, time), block) in cache.iter().zip(blocks.iter().filter(|b| !b.e_only)) {
                    let _ = writeln!(
                        file,
                        "line {} {:.6} dist {:.6} accel {:.3}",
                        id, time, block.distance, block.acceleration
                    );
                }
            }
        }
        Self {
            total: cumulative + trailing_delay,
            prepare,
            elapsed,
        }
    }

    fn elapsed_at(&self, line: usize) -> Option<f64> {
        self.elapsed.get(line).copied().flatten()
    }
}

/// Upstream discretizes G2/G3 into `segments` internal G1s during
/// processing (`GCodeProcessor::process_G2_G3`), each incrementing the g1
/// line counter; the FILE still carries the single arc line, so the export
/// advances the counter by `1 + internal`. Segment count per flavor:
/// - MarlinFirmware: `plan_arc` (`GCodeProcessor.cpp:1690-1700`)
/// - otherwise: `ArcWelder::arc_discretization_steps` at 0.0125 tolerance
///   (`ArcWelder.hpp:48-64`)
fn arc_internal_g1_lines(code: &str, command: &str, state: &MotionState) -> usize {
    let i = word(code, 'I').unwrap_or(0.0);
    let j = word(code, 'J').unwrap_or(0.0);
    let radius = (i * i + j * j).sqrt();
    let (end_x, end_y) = (word(code, 'X'), word(code, 'Y'));
    let start = state.position;
    let end_x = end_x.map_or(start[0], |value| {
        if state.relative {
            start[0] + value
        } else {
            value
        }
    });
    let end_y = end_y.map_or(start[1], |value| {
        if state.relative {
            start[1] + value
        } else {
            value
        }
    });
    let rel_start = (-i, -j);
    let rel_end = (end_x - (start[0] + i), end_y - (start[1] + j));
    let mut angle = (rel_start.0 * rel_end.1 - rel_start.1 * rel_end.0)
        .atan2(rel_start.0 * rel_end.0 + rel_start.1 * rel_end.1);
    if angle < 0.0 {
        angle += std::f64::consts::TAU;
    }
    if command == "G2" {
        angle -= std::f64::consts::TAU;
    }
    let angle = angle.abs().min(std::f64::consts::TAU);
    let feedrate_mm_s = word(code, 'F').map_or(state.feedrate, |value| value / 60.0) as f32;
    let segments = if state.gcode_flavor == crate::GCodeFlavor::MarlinFirmware {
        const MAX_ARC_DEVIATION: f32 = 0.02;
        const MIN_ARC_SEGMENTS_PER_SEC: f32 = 50.0;
        const MIN_ARC_SEGMENT_MM: f32 = 0.1;
        const MAX_ARC_SEGMENT_MM: f32 = 2.0;
        let radius_mm = radius as f32;
        let segment_mm = ((8.0 * radius_mm * MAX_ARC_DEVIATION)
            .sqrt()
            .min(feedrate_mm_s / MIN_ARC_SEGMENTS_PER_SEC))
        .clamp(MIN_ARC_SEGMENT_MM, MAX_ARC_SEGMENT_MM);
        let flat_mm = radius_mm * angle as f32;
        ((flat_mm / segment_mm + 0.8) as usize).max(1)
    } else {
        const GCODE_ARC_TOLERANCE: f64 = 0.0125;
        let d = radius - GCODE_ARC_TOLERANCE;
        if d < f64::EPSILON {
            // Radius smaller than the deviation: acute angles interpolate
            // with one segment; obtuse angles test the center distance.
            if angle < std::f64::consts::PI
                || radius * (1.0 + (std::f64::consts::PI - 0.5 * angle).cos()) < GCODE_ARC_TOLERANCE
            {
                1
            } else {
                2
            }
        } else {
            (angle / (2.0 * (d / radius).acos())).ceil() as usize
        }
        .max(1)
    };
    segments.saturating_sub(1)
}

fn scheduled_times(blocks: &[MotionBlock], events: &[FlushEvent]) -> (Vec<f64>, f64) {
    let mut planner = RollingPlanner::new();
    let mut times = vec![0.0; blocks.len()];
    let mut pending = VecDeque::new();
    let mut pushed = 0;
    let mut emitted = 0;

    for event in events {
        while pushed < event.block_count {
            let batch = planner.push(&blocks[pushed]);
            pushed += 1;
            record_batch(blocks, &mut times, &mut emitted, batch, &mut pending);
        }
        if let Some(delay) = event.delay.filter(|delay| delay.seconds > 0.0) {
            if let Some(last) = pending
                .back_mut()
                .filter(|last| last.target == delay.target)
            {
                last.seconds += delay.seconds;
            } else {
                pending.push_back(delay);
            }
        }
        let batch = planner.flush();
        record_batch(blocks, &mut times, &mut emitted, batch, &mut pending);
    }
    while pushed < blocks.len() {
        let batch = planner.push(&blocks[pushed]);
        pushed += 1;
        record_batch(blocks, &mut times, &mut emitted, batch, &mut pending);
    }
    let batch = planner.finish();
    record_batch(blocks, &mut times, &mut emitted, batch, &mut pending);

    let trailing_delay = pending
        .into_iter()
        .fold(0.0_f32, |total, delay| total + delay.seconds);
    (times, f64::from(trailing_delay))
}

fn record_batch(
    blocks: &[MotionBlock],
    times: &mut [f64],
    emitted: &mut usize,
    mut batch: Vec<f64>,
    pending: &mut VecDeque<PendingDelay>,
) {
    for (block, time) in blocks[*emitted..].iter().zip(&mut batch) {
        let Some(delay) = pending.front() else {
            break;
        };
        if delay.target == DelayTarget::Any
            || delay.target == DelayTarget::ToolChange && block.kind == MotionKind::ToolChange
        {
            *time = f64::from(*time as f32 + delay.seconds);
            pending.pop_front();
        }
    }
    let end = *emitted + batch.len();
    times[*emitted..end].copy_from_slice(&batch);
    *emitted = end;
}

fn synchronizes_planner(code: &str, measured_g29: bool) -> bool {
    let command = code.split_whitespace().next().unwrap_or_default();
    matches!(command, "M0" | "M1")
        || matches!(command, "G4" | "M400")
            && (word(code, 'S').is_some() || word(code, 'P').is_some())
        || measured_g29
        || command == "M191" && word(code, 'S').unwrap_or(0.0) > 40.0
        || command == "G92" && word(code, 'E').is_none()
        || command == "M702" && code.contains('C')
        || command == "SYNC" && word(code, 'T').is_some()
}

#[cfg(test)]
mod tests;

fn tool_id(code: &str) -> Option<u8> {
    let command = code.split_whitespace().next()?;
    command.strip_prefix('T')?.parse().ok()
}

fn selects_initial_tool(code: &str, active_tool: &mut Option<u8>) -> bool {
    let Some(tool) = tool_id(code) else {
        return false;
    };
    active_tool.replace(tool).is_none()
}
