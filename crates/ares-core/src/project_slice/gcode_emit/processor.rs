mod arc_accounting;
mod delays;
mod envelope;
mod estimate;
mod motion;
mod motion_util;
mod schedule;
mod stats;
mod time;
use crate::options::GCodeFlavor;
use estimate::Estimate;
pub(super) use stats::{PRINT_TIME_SEC_PLACEHOLDER, USED_FILAMENT_LENGTH_PLACEHOLDER};
use time::{duration, minutes};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ProcessorLimits {
    pub(super) print_acceleration: f64,
    pub(super) retract_acceleration: f64,
    pub(super) travel_acceleration: f64,
    pub(super) gcode_flavor: GCodeFlavor,
    pub(super) bbl_printer: bool,
    pub(super) junction_deviation: f64,
    pub(super) max_feedrate: [f64; 4],
    pub(super) max_acceleration: [f64; 4],
    pub(super) jerk: [f64; 4],
}

#[cfg(test)]
impl Default for ProcessorLimits {
    fn default() -> Self {
        Self {
            print_acceleration: 0.0,
            retract_acceleration: 0.0,
            travel_acceleration: 0.0,
            gcode_flavor: GCodeFlavor::default(),
            bbl_printer: false,
            junction_deviation: 0.0,
            max_feedrate: [0.0; 4],
            max_acceleration: [0.0; 4],
            jerk: [9.0, 9.0, 3.0, 2.5],
        }
    }
}

pub(super) fn process(
    mut output: Vec<u8>,
    emit_progress: bool,
    machine_load_filament_time: f64,
    used_filament: f64,
    limits: ProcessorLimits,
) -> Vec<u8> {
    let text = String::from_utf8(std::mem::take(&mut output)).expect("generated G-code is UTF-8");
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
        // Upstream computes the progress pair entirely in f32
        // (`GCodeProcessor.cpp:1251-1252`): `int(100.0f * elapsed_time /
        // machine.time)` over the float `machine.time`, and
        // `time_in_minutes(machine.time - elapsed_time)` on float operands —
        // f64 arithmetic flips boundary crossings against the float cache.
        let total = estimate.total as f32;
        let percent = if total > 0.0 {
            (100.0_f32 * elapsed / total) as u64
        } else {
            0
        };
        let remaining = minutes(f64::from(total - elapsed));
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

#[cfg(test)]
mod arc_cache_tests;
#[cfg(test)]
mod envelope_replay_tests;
#[cfg(test)]
mod seam_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod trapezoid_tests;
#[cfg(test)]
mod zero_acceleration_tests;
