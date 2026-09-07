//! `GCodeProcessor` motion accounting and exported progress-time lookup.

use super::arc_accounting::arc_internal_g1_lines;
use super::delays::command_delay;
use super::motion::{MotionBlock, MotionKind, MotionState};
use super::motion_util::word;
use super::schedule::{DelayTarget, FlushEvent, PendingDelay, scheduled_times};
use super::{ProcessorLimits, is_progress_motion};

pub(super) struct Estimate {
    pub(super) total: f64,
    pub(super) prepare: f64,
    elapsed: Vec<Option<f64>>,
}
impl Estimate {
    pub(super) fn from_lines(
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
        let mut cumulative = 0.0;
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
        Self {
            total: cumulative + trailing_delay,
            prepare,
            elapsed,
        }
    }

    pub(super) fn elapsed_at(&self, line: usize) -> Option<f64> {
        self.elapsed.get(line).copied().flatten()
    }
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
