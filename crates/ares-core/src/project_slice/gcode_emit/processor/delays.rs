use super::motion_util::word;

pub(super) fn command_delay(code: &str) -> Option<f64> {
    if code.starts_with("M400") {
        return Some(word(code, 'S').unwrap_or(0.0) + word(code, 'P').unwrap_or(0.0) * 0.001);
    }
    // OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:4859-4864.
    if code.starts_with("G29") && !code.starts_with("G29.") {
        return Some(260.0);
    }
    // OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:5150-5157.
    if code.starts_with("M191") && word(code, 'S').unwrap_or(0.0) > 40.0 {
        return Some(720.0);
    }
    None
}
