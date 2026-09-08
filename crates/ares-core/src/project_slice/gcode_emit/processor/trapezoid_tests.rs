//! `GCodeProcessor.cpp:255-274` `calculate_trapezoid` distance-based times.
use super::motion::planned_trapezoid_time;

#[test]
fn wrong_direction_decel_distance_contributes_zero_time_not_negative() {
    // BIQU B1 END-code `G1 E-2 Z0.2 F2400` block state (wave10 dump):
    // the safe (exit) feedrate exceeds cruise, so upstream's
    // `estimated_acceleration_distance(cruise, exit, -acceleration)`
    // clamps to a zero deceleration distance and the decel time is 0.
    // The velocity-difference shortcut subtracted `(cruise-exit)/accel`
    // = -0.04 s instead.
    let time = planned_trapezoid_time(0.2, 6.0, 0.3, 10.0, 100.0);
    // accel (6-0.3)/100 + cruise (0.2-0.17955)/6, no decel term.
    assert!((time - 0.060408).abs() < 1e-5, "{time}");
}

#[test]
fn wrong_direction_accel_distance_contributes_zero_time_not_negative() {
    // Mirrored case: entry above cruise means a zero acceleration
    // distance upstream, never a negative acceleration time.
    let time = planned_trapezoid_time(0.2, 6.0, 10.0, 0.3, 100.0);
    assert!((time - 0.060408).abs() < 1e-5, "{time}");
}
