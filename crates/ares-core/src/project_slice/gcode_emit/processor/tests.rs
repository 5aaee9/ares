use super::estimate::Estimate;
use super::motion::{MotionBlock, MotionState, planned_times};
use super::{ProcessorLimits, process};

// Synthetic timing tests isolate other behavior with nonbinding axis caps;
// zero is a real acceleration limit, not an unlimited sentinel.
fn nonbinding_axis_limits() -> ProcessorLimits {
    ProcessorLimits {
        max_acceleration: [f64::MAX; 4],
        ..ProcessorLimits::default()
    }
}

// The synthetic footer fixtures use the BBL placeholder set; the end-to-end
// suite covers the compatible set via Orca parity.
fn bbl_limits() -> ProcessorLimits {
    ProcessorLimits {
        bbl_printer: true,
        ..nonbinding_axis_limits()
    }
}

#[test]
fn inserts_progress_and_rewrites_time_fields() {
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM204 S1000\nG1 X1000 F600\nM73 P100 R0\n".to_vec();
    let output = String::from_utf8(process(output, true, 0.0, 0.0, bbl_limits())).unwrap();
    assert!(output.contains("total estimated time: 1m 40s"), "{output}");
    assert!(output.contains("M73 P0 R"));
    assert!(output.contains("; model printing time:"));
    assert!(!output.contains("total estimated time: 0s"));
}

#[test]
fn rewrites_compatible_time_footer_for_non_bbl_printers() {
    let output = b"; estimated printing time (normal mode) = 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nG1 X1000 F600\nM73 P100 R0\n"
        .to_vec();

    let output =
        String::from_utf8(process(output, true, 0.0, 0.0, nonbinding_axis_limits())).unwrap();

    assert!(
        output.contains("; estimated printing time (normal mode) = 1m 40s"),
        "{output}"
    );
    assert!(!output.contains("model printing time"), "{output}");
}

#[test]
fn disable_m73_suppresses_synthetic_progress_lines() {
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM204 S1000\nG1 X1000 F600\nM73 P100 R0\n"
        .to_vec();

    let output = String::from_utf8(process(output, false, 0.0, 0.0, bbl_limits())).unwrap();

    assert!(!output.lines().any(|line| line.starts_with("M73 P")));
    assert!(output.contains("total estimated time: 1m 40s"));
}

#[test]
fn progress_updates_follow_motion_lines_not_delay_commands() {
    let output = b"M73 P0 R0\nT0\nG1 X1 F60\nM73 P100 R0\n".to_vec();

    let output = String::from_utf8(process(output, true, 120.0, 0.0, bbl_limits())).unwrap();

    assert!(output.contains("T0\nG1 X1 F60\nM73 P"), "{output}");
    assert!(!output.contains("T0\nM73 P"), "{output}");
}
#[test]
fn finalized_motion_time_is_exported_after_the_next_motion_command() {
    let output =
        b"M73 P0 R0\nM204 S1000\nG1 X1 F60\nM400\nG1 X2 F60\nM622 J1\nG29 A1\nM73 P100 R0\n"
            .to_vec();

    let output = String::from_utf8(process(output, true, 0.0, 0.0, bbl_limits())).unwrap();

    assert!(output.contains("G1 X2 F60\nM73 P99 R0\nM622"), "{output}");
}
#[test]
fn non_bbl_g29_counts_the_bed_leveling_delay_without_m622_markers() {
    let output = b"M73 P0 R0\nG28\nG29 ;Auto bed leveling detecting\nM109 S220\nM204 S1000\nG1 X600 F3600\nM73 P100 R0\n"
        .to_vec();

    let output =
        String::from_utf8(process(output, true, 0.0, 0.0, nonbinding_axis_limits())).unwrap();

    assert!(output.contains("M73 P0 R4\n"), "{output}");
}

#[test]
fn bbl_g29_counts_the_bed_leveling_delay_only_inside_m622_j1() {
    let gated = b"M73 P0 R0\nM622 J1\nG29\nM623\nM204 S1000\nG1 X600 F3600\nM73 P100 R0\n".to_vec();
    let ungated = b"M73 P0 R0\nG29\nM204 S1000\nG1 X600 F3600\nM73 P100 R0\n".to_vec();

    let measured =
        String::from_utf8(process(gated.to_vec(), true, 0.0, 0.0, bbl_limits())).unwrap();
    let skipped = String::from_utf8(process(ungated, true, 0.0, 0.0, bbl_limits())).unwrap();

    assert!(measured.contains("M73 P0 R4\n"), "{measured}");
    assert!(skipped.contains("M73 P0 R0\n"), "{skipped}");
}

#[test]
fn g4_dwell_s_word_adds_seconds_to_the_total_time() {
    // OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:4848-4856: process_G4
    // passes S seconds to simulate_st_synchronize.
    let plain = ["M204 S1000", "G1 X600 F3600", "G1 X1200 F3600"].map(str::to_owned);
    let dwell = ["M204 S1000", "G1 X600 F3600", "G4 S10", "G1 X1200 F3600"].map(str::to_owned);
    let plain = Estimate::from_lines(&plain, 0.0, nonbinding_axis_limits());
    let dwell = Estimate::from_lines(&dwell, 0.0, nonbinding_axis_limits());

    assert!(
        (dwell.total - plain.total - 10.0).abs() < 1e-6,
        "{}",
        dwell.total
    );
}

#[test]
fn g4_dwell_p_word_counts_milliseconds_and_s_takes_precedence() {
    let p = ["M204 S1000", "G1 X600 F3600", "G4 P500", "G1 X1200 F3600"].map(str::to_owned);
    let s_and_p = [
        "M204 S1000",
        "G1 X600 F3600",
        "G4 S10 P500",
        "G1 X1200 F3600",
    ]
    .map(str::to_owned);

    let p = Estimate::from_lines(&p, 0.0, nonbinding_axis_limits());
    let s_and_p = Estimate::from_lines(&s_and_p, 0.0, nonbinding_axis_limits());

    // process_G4's has_value('S') || has_value('P') short-circuits, so a
    // parseable S suppresses the P lookup entirely. The 2e-6 tolerance covers
    // the f32 block-time attribution of the appended dwell seconds.
    assert!((p.total - 20.543351).abs() < 2e-6, "{}", p.total);
    assert!(
        (s_and_p.total - 30.043351).abs() < 2e-6,
        "{}",
        s_and_p.total
    );
}

#[test]
fn g4_dwell_shifts_progress_markers_and_the_total_footer() {
    let output = b"; estimated printing time (normal mode) = 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM204 S1000\nG1 X600 F3600\nG4 S10\nG1 X1200 F3600\nM73 P100 R0\n"
        .to_vec();

    let output =
        String::from_utf8(process(output, true, 0.0, 0.0, nonbinding_axis_limits())).unwrap();

    assert!(output.contains("G1 X1200 F3600\nM73 P66 R0\n"), "{output}");
    assert!(
        output.contains("; estimated printing time (normal mode) = 30s"),
        "{output}"
    );
}

#[test]
fn preparation_time_ends_at_first_print_feature() {
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\n; FEATURE: Custom\nM204 S1000\nG1 X600 F3600\n; FEATURE: Inner wall\nG1 X1200 F3600\nM73 P100 R0\n".to_vec();

    let output = String::from_utf8(process(output, false, 0.0, 0.0, bbl_limits())).unwrap();

    assert!(
        output.contains("estimated first layer printing time (normal mode) = 10s"),
        "{output}"
    );
}

#[test]
fn collinear_cruise_time_is_not_zeroed_by_default_jerk() {
    let mut state = MotionState {
        max_acceleration: [f64::MAX; 4],
        ..MotionState::default()
    };
    state.motion("M204 S1000");
    let first = state.motion("G1 X600 F3600").unwrap();
    let second = state.motion("G1 X1200 F3600").unwrap();

    let times = planned_times(&[first, second]);

    assert!((times.iter().sum::<f64>() - 20.043_349_266_052_246).abs() < 1e-6);
}

#[test]
fn tracks_relative_e_only_moves() {
    let mut state = MotionState::default();
    state.motion("M83");
    let block = state.motion("G1 E-.4 F1800").unwrap();
    assert!((block.distance - 0.4).abs() < 1e-6, "{}", block.distance);
}

#[test]
fn legacy_m204_s_sets_travel_and_t_as_retract() {
    let mut state = MotionState::default();
    state.motion("M204 S500 T125");
    assert_eq!(state.acceleration, 500.0);
    assert_eq!(state.travel_acceleration, 500.0);
    assert_eq!(state.retract_acceleration, 125.0);
}

#[test]
fn machine_max_feedrate_limits_extrusion_time() {
    let output = b"; model printing time: 0s; total estimated time: 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\nM203 E30\nM204 R1000\nM83\nG1 E60 F3600\nM73 P100 R0\n"
        .to_vec();

    let output = String::from_utf8(process(output, false, 0.0, 0.0, bbl_limits())).unwrap();

    assert!(output.contains("total estimated time: 2s"), "{output}");
}

#[test]
fn machine_max_acceleration_limits_motion_block() {
    let mut state = MotionState::default();
    state.motion("M201 E100");
    state.motion("M204 R1000");
    state.motion("M83");

    let block = state.motion("G1 E10 F3600").unwrap();

    assert_eq!(block.acceleration, 100.0);
}

#[test]
fn m204_updates_respect_machine_acceleration_envelopes() {
    let mut state = MotionState::with_acceleration_limits(20_000.0, 30_000.0, 9_000.0);

    state.motion("M204 P20000 R30000 T20000");

    assert_eq!(state.acceleration, 20_000.0);
    assert_eq!(state.retract_acceleration, 30_000.0);
    assert_eq!(state.travel_acceleration, 9_000.0);
}

#[test]
fn travel_blocks_retain_print_acceleration_for_centripetal_limits() {
    let mut state = MotionState {
        max_acceleration: [f64::MAX; 4],
        ..MotionState::default()
    };
    state.motion("M204 P500 T10000");

    let block = state.motion("G1 X10 F6000").unwrap();

    assert_eq!(block.acceleration, 10_000.0);
    assert_eq!(block.centripetal_acceleration, 500.0);
}

#[test]
fn collinear_blocks_keep_speed_at_the_shared_junction() {
    let block = || MotionBlock {
        distance: 10.0,
        speed: 10.0,
        acceleration: 100.0,
        centripetal_acceleration: 100.0,
        jerk: [10.0; 4],
        direction: [1.0, 0.0, 0.0, 0.0],
        kind: super::motion::MotionKind::Regular,
        e_only: false,
    };

    let elapsed = planned_times(&[block(), block()]).into_iter().sum::<f64>();

    assert!((elapsed - 2.0).abs() < 1e-9, "{elapsed}");
}

#[test]
fn tool_change_block_resets_the_following_junction() {
    let tool_change = MotionBlock {
        distance: 0.0,
        speed: 0.0,
        acceleration: 0.0,
        centripetal_acceleration: 0.0,
        jerk: [0.0; 4],
        direction: [0.0; 4],
        kind: super::motion::MotionKind::ToolChange,
        e_only: false,
    };
    let retract = MotionBlock {
        distance: 3.0,
        speed: 30.0,
        acceleration: 30_000.0,
        centripetal_acceleration: 10_000.0,
        jerk: [9.0, 9.0, 3.0, 2.5],
        direction: [0.0, 0.0, 0.0, -1.0],
        kind: super::motion::MotionKind::Regular,
        e_only: false,
    };

    let times = planned_times(&[tool_change, retract]);

    assert_eq!(times[0], 0.0);
    assert!((times[1] - 0.100920171).abs() < 1e-7, "{}", times[1]);
}

#[test]
fn isolated_block_uses_firmware_safe_entry_speed() {
    let block = MotionBlock {
        distance: 10.0,
        speed: 10.0,
        acceleration: 100.0,
        centripetal_acceleration: 100.0,
        jerk: [9.0, 9.0, 3.0, 2.5],
        direction: [1.0, 0.0, 0.0, 0.0],
        kind: super::motion::MotionKind::Regular,
        e_only: false,
    };

    let elapsed = planned_times(&[block])[0];

    assert!((elapsed - 1.001).abs() < 1e-6, "{elapsed}");
}

#[test]
fn single_block_synchronization_waits_for_next_motion() {
    let lines = ["M204 S1000", "G1 X600 F3600", "M1", "G1 X1200 F3600"].map(str::to_owned);

    let estimate = Estimate::from_lines(&lines, 0.0, nonbinding_axis_limits());

    assert!(
        (estimate.total - 20.043_349_266_052_246).abs() < 1e-6,
        "{}",
        estimate.total
    );
}

#[test]
fn initial_tool_selection_adds_machine_load_time_once() {
    let lines = ["T0 H-1", "T0 H-1"].map(str::to_owned);

    let estimate = Estimate::from_lines(&lines, 29.0, ProcessorLimits::default());

    assert_eq!(estimate.total, 29.0);
}
#[test]
fn spiral_arc_p_one_is_one_turn_at_same_endpoint() {
    let mut state = MotionState::default();
    let block = state.motion("G3 Z.6 I1 J0 P1 F600").unwrap();
    assert!(
        (block.distance - (2.0 * std::f64::consts::PI).hypot(0.6)).abs() < 1e-6,
        "{}",
        block.distance
    );
}

#[test]
fn marlin_arc_is_discretized_into_firmware_segments() {
    let mut state = MotionState::default();

    let blocks = state.motions("G3 X0 Y2 I0 J1 F600");

    assert_eq!(blocks.len(), 10);
    let distance = blocks.iter().map(|block| block.distance).sum::<f64>();
    assert!((distance - 3.123189).abs() < 1e-6, "{distance}");
}

#[test]
fn homing_command_emits_motion_to_requested_axes() {
    let mut state = MotionState::default();
    state.motion("G1 X10 Y20 Z3 F600");

    let block = state.motion("G28 X").unwrap();

    assert!((block.distance - 10.0).abs() < 1e-9);
    assert_eq!(state.position, [0.0, 20.0, 3.0]);
}

#[test]
fn homing_motion_contributes_to_total_estimate() {
    let lines = ["G1 X10 F600", "G28 X"].map(str::to_owned);

    let estimate = Estimate::from_lines(&lines, 0.0, nonbinding_axis_limits());

    assert!(
        (estimate.total - 2.000_159_978_866_577).abs() < 1e-9,
        "{}",
        estimate.total
    );
}

#[test]
fn unsupported_commands_do_not_change_motion_feedrate() {
    let mut state = MotionState::default();
    state.motion("G1 X1 F600");
    state.motion("G130 F4.36536");

    let block = state.motion("G1 X2").unwrap();

    // F600 through the f32 reciprocal lands at 10.000001 mm/s
    // (`GCodeProcessor.cpp:41`).
    assert_eq!(block.speed, 10.000_000_953_674_316);
}
#[test]
fn arc_p_word_adds_full_turns() {
    let mut state = MotionState::default();
    let block = state.motion("G3 X0 Y2 I0 J1 P1 F600").unwrap();
    assert!((block.distance - 3.0 * std::f64::consts::PI).abs() < 1e-9);
}

#[test]
fn bare_g92_resets_all_logical_axes() {
    let mut state = MotionState {
        position: [10.0, 20.0, 30.0],
        e_position: 40.0,
        ..MotionState::default()
    };

    assert!(state.motion("G92").is_none());

    assert_eq!(state.position, [0.0; 3]);
    assert_eq!(state.e_position, 0.0);
}
#[test]
fn progress_skips_e_only_retract_lines() {
    // E-only moves have no g1_times_cache entry of their own (their ids are
    // absent from Orca's dump). An M73 after the retract carries the previous
    // motion line's cumulative time, and the line after the retract sees no
    // entry, so the retract's own cumulative time never reaches an M73.
    let output = b"M73 P0 R0\nG1 X100 F600\nG1 E-5 F300\nG1 Z.8 F600\nM73 P100 R0\n".to_vec();

    let output =
        String::from_utf8(process(output, true, 0.0, 0.0, nonbinding_axis_limits())).unwrap();

    assert!(output.contains("G1 E-5 F300\nM73 P"), "{output}");
    // the only M73 after Z.8 is the final P100 placeholder, so no emission
    // carries the retract's own cumulative time
    assert!(output.ends_with("G1 Z.8 F600\nM73 P100 R0\n"), "{output}");
}

// `get_time_dhms` prints sub-second times with the `%f` format
// (`Utils.hpp:540-560`): values at or below one second keep six fractional
// digits instead of truncating to `0s`.
#[test]
fn first_layer_time_trailer_prints_fractional_seconds() {
    let output = b"; estimated printing time (normal mode) = 0s\n; estimated first layer printing time (normal mode) = 0s\nM73 P0 R0\n;TYPE:Custom\nG1 X5 F600\n;TYPE:Inner wall\nG1 X1000 F600\nM73 P100 R0\n".to_vec();

    let output =
        String::from_utf8(process(output, true, 0.0, 0.0, nonbinding_axis_limits())).unwrap();

    assert!(
        output.contains("; estimated first layer printing time (normal mode) = 0.500040s"),
        "{output}"
    );
}

// A full-circle arc (`is_full_circle`, `GCodeProcessor.cpp:4660-4669`) pins the
// sweep to a full turn before the clockwise adjustment, so the ArcWelder
// discretization consumes one g1 line id per segment.
#[test]
fn full_circle_arc_consumes_discretized_g1_line_ids() {
    use super::arc_accounting::arc_internal_g1_lines;
    use super::motion::MotionState;
    let state = MotionState {
        position: [0.0, 0.0, 0.0],
        ..MotionState::default()
    };
    let internal = arc_internal_g1_lines("G3 Z.5 I5 J0 P1 F600", "G3", &state);
    assert_eq!(internal, 44, "radius 5 full circle at 0.0125 tolerance");
}

// `fast_float::from_chars` rejects a leading `+` (`GCodeReader.cpp:276-288`),
// so `Z+0.5` words carry no axis value at all.
#[test]
fn leading_plus_word_carries_no_value() {
    let mut state = MotionState::default();
    let blocks = state.motions("G1 X+10 F600");
    assert!(blocks.is_empty());
    assert_eq!(state.position, [0.0, 0.0, 0.0]);
}

// `m_feedrate = line.f() * MMMIN_TO_MMSEC` multiplies by the f32 reciprocal
// (`GCodeProcessor.cpp:41`), so F21000 lands at 350.000031 mm/s, not 350.0.
#[test]
fn feedrate_converts_through_f32_reciprocal() {
    use super::motion::MotionState;
    let mut state = MotionState::default();
    let _ = state.motions("G1 F21000");
    assert_eq!(state.feedrate, 350.000_030_517_578_1);
}
