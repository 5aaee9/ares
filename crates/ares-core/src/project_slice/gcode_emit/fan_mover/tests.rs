use super::FanMover;
use crate::GCodeFlavor;

fn mover(delay: f64, kickstart: f64) -> FanMover {
    FanMover::new(delay, kickstart, false, false, GCodeFlavor::MarlinLegacy)
}

#[test]
fn pass_through_when_disabled() {
    let mut fan_mover = mover(0.0, 0.0);
    let gcode = "G1 X10 F6000\nM106 S255\nG1 X20\n";
    let out = fan_mover.process_gcode(gcode, true);
    assert_eq!(out, gcode);
}

#[test]
fn ramp_up_command_delayed_into_buffer() {
    // A fan-up command moves back into the buffer by the delay window.
    let mut fan_mover = mover(1.0, 0.0);
    let out = fan_mover.process_gcode(
        "G1 F6000\nM106 S255\nG1 X10\nG1 X10\nG1 X10\nG1 X10\n",
        true,
    );
    // The fan line must not stay at its original position; it lands inside
    // the buffered motion window (output order: motion, fan, motion...).
    assert!(!out.starts_with("G1 F6000\nM106"));
    assert!(out.contains("M106 S255"));
}

#[test]
fn already_flushed_fan_prints_directly_on_ramp_up() {
    // The S128 was printed directly (buffer too short to hold it), so the
    // later full-speed command also prints directly — both survive, as
    // upstream's short-buffer path does.
    let mut fan_mover = mover(1.0, 0.0);
    let out = fan_mover.process_gcode(
        "G1 F6000\nM106 S128\nG1 X100\nM106 S255\nG1 X100\nG1 X100\nG1 X100\n",
        true,
    );
    assert_eq!(out.matches("M106").count(), 2);
}

#[test]
fn kickstart_full_speed_precedes_eventual_target() {
    // Long-enough motion window for the kickstart target to land via the
    // buffer: S255 kickstart first, the S128 target inside the window.
    let mut fan_mover = mover(0.0, 0.5);
    let out = fan_mover.process_gcode(
        "G1 F6000\nM106 S128\nG1 X100\nG1 X100\nG1 X100\nG1 X100\nG1 X100\n",
        true,
    );
    let full = out.find("M106 S255");
    let target = out.find("M106 S128");
    assert!(full.is_some(), "kickstart line missing: {out}");
    assert!(target.is_some(), "target line missing: {out}");
    assert!(full < target);
}

#[test]
fn kickstart_target_lands_half_second_later() {
    // Regression (CONSTRUCT3D Construct 1, fan_speedup_time=1, fan_kickstart=0.5):
    // the kickstart target M106 lands 0.5s after the direct kickstart print —
    // after the third wall move — not immediately after the travel.
    let mut fan_mover = mover(1.0, 0.5);
    let out = fan_mover.process_gcode(
        "G1 X108.927 Y134.04 F3000\nM106 S255\nG92 E0\nG1 E-.49 F3000\nG1 F6000\nG1 X109.005 Y134.04 E-.01632\nM204 P4000\nG3 Z.66 I-.056 J.606 P1 F19200\nG1 X117.3 Y134.8 Z.66\nG1 Z.46\nG1 E.7 F3000\nG1 F3763\nG1 X107.7 Y134.8 E.20259\nG1 X107.7 Y125.2 E.20259\nG1 X117.3 Y125.2 E.20259\nG1 X117.3 Y134.76 E.20174\n",
        true,
    );
    let second = out
        .match_indices("M106 S255")
        .nth(1)
        .map(|(i, _)| i)
        .unwrap();
    let anchor = out.find("G1 X117.3 Y125.2 E.20259").unwrap();
    let next_wall = out.find("G1 X117.3 Y134.76 E.20174").unwrap();
    assert!(
        second > anchor && second < next_wall,
        "kickstart target misplaced:\n{out}"
    );
}
