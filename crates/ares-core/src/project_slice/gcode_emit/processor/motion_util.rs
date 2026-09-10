//! G-code word parsing and kinematic clamp helpers shared by motion planning.

/// Parse a G-code word (letter + numeric value) from a command string.
/// A leading `+` carries no value: `fast_float::from_chars` rejects it
/// (`GCodeReader.cpp:276-288`), so upstream treats `Z+0.5` as axis-less.
pub(super) fn word(code: &str, letter: char) -> Option<f64> {
    let start = code.find(letter)? + letter.len_utf8();
    let value = &code[start..];
    let end = value
        .find(|character: char| character.is_ascii_alphabetic())
        .unwrap_or(value.len());
    let value = value[..end].trim();
    if value.starts_with('+') {
        return None;
    }
    value.parse::<f32>().ok().map(f64::from)
}

pub(super) fn clamped_word(code: &str, letter: char, current: f64, maximum: f64) -> f64 {
    word(code, letter).map_or(current, |value| clamp(value, maximum))
}

/// Parse a `KEY=value` assignment such as Klipper's `SET_VELOCITY_LIMIT
/// ACCEL=500 ACCEL_TO_DECEL=250`.
pub(super) fn assignment(code: &str, key: &str) -> Option<f64> {
    let start = code.find(key)? + key.len();
    let value = &code[start..];
    let end = value
        .find(|character: char| !character.is_ascii_digit() && character != '.')
        .unwrap_or(value.len());
    value[..end].trim().parse::<f32>().ok().map(f64::from)
}

/// `GCodeProcessor.cpp:41`: the feed-rate conversion multiplies by the f32
/// reciprocal, not by dividing by 60.
pub(super) const MMMIN_TO_MMSEC: f32 = 1.0 / 60.0;

pub(super) fn clamp(value: f64, maximum: f64) -> f64 {
    if maximum > 0.0 {
        value.min(maximum)
    } else {
        value
    }
}

pub(super) fn norm(value: [f64; 4]) -> f64 {
    value
        .iter()
        .map(|component| component * component)
        .sum::<f64>()
        .sqrt()
}

pub(super) fn scale(value: [f64; 4], factor: f64) -> [f64; 4] {
    [
        value[0] * factor,
        value[1] * factor,
        value[2] * factor,
        value[3] * factor,
    ]
}
