use super::motion::MotionState;
use super::motion_util::word;

/// Upstream discretizes G2/G3 into `segments` internal G1s during
/// processing (`GCodeProcessor::process_G2_G3`), each incrementing the g1
/// line counter; the FILE still carries the single arc line, so the export
/// advances the counter by `1 + internal`. Segment count per flavor:
/// - MarlinFirmware: `plan_arc` (`GCodeProcessor.cpp:1690-1700`)
/// - otherwise: `ArcWelder::arc_discretization_steps` at 0.0125 tolerance
///   (`ArcWelder.hpp:48-64`)
pub(super) fn arc_internal_g1_lines(code: &str, command: &str, state: &MotionState) -> usize {
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
