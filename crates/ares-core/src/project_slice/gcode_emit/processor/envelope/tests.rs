use super::ProcessorLimits;
use crate::project_slice::gcode_emit::processor::motion::MotionState;
use crate::{GCodeFlavor, MachineEnvelopeOptions};

fn afinia() -> MachineEnvelopeOptions {
    // Normal/stealth arrays from the complete actual-AppImage case-oKLMEh export.
    serde_json::from_value(serde_json::json!({
        "machine_max_speed_x": [500, 200], "machine_max_speed_y": [500, 200],
        "machine_max_speed_z": [12, 12], "machine_max_speed_e": [25, 25],
        "machine_max_acceleration_x": [20000, 20000],
        "machine_max_acceleration_y": [20000, 20000],
        "machine_max_acceleration_z": [500, 200],
        "machine_max_acceleration_e": [5000, 5000],
        "machine_max_acceleration_extruding": [20000, 20000],
        "machine_max_acceleration_retracting": [5000, 5000],
        "machine_max_acceleration_travel": [20000, 20000],
        "machine_max_jerk_x": [9, 9], "machine_max_jerk_y": [9, 9],
        "machine_max_jerk_z": [0.2, 0.4], "machine_max_jerk_e": [2.5, 2.5],
        "machine_max_junction_deviation": [0.01]
    }))
    .unwrap()
}

#[test]
fn afinia_first_retract_and_z_use_config_without_emitted_envelope() {
    let limits = ProcessorLimits::from_config(&afinia(), GCodeFlavor::Klipper, false);
    let mut state = MotionState::with_limits(limits);
    state.motion("M83");
    let retract = state.motion("G1 E-.8 F1800").unwrap();
    assert_eq!(retract.speed, 25.0);
    assert_eq!(retract.acceleration, 5000.0);
    assert_eq!(retract.jerk, [9.0, 9.0, f64::from(0.2_f32), 2.5]);
    state.motion("SET_VELOCITY_LIMIT ACCEL=500 ACCEL_TO_DECEL=250");
    state.motion("SET_VELOCITY_LIMIT ACCEL=4000 ACCEL_TO_DECEL=2000");
    let z = state.motion("G1 Z.6 F12000").unwrap();
    assert_eq!(z.speed, 12.0);
    assert_eq!(z.acceleration, 500.0);
    assert_eq!(state.max_travel_acceleration, 0.0);
    assert_eq!(state.travel_acceleration, 4000.0);
}

#[test]
fn artillery_second_displacement_is_z_limited_not_travel_config_limited() {
    let machine = serde_json::from_value(serde_json::json!({
        "emit_machine_limits_to_gcode": false,
        "machine_max_speed_z": [20, 12], "machine_max_speed_e": [30, 120],
        "machine_max_acceleration_x": [20000, 1000],
        "machine_max_acceleration_y": [20000, 1000],
        "machine_max_acceleration_extruding": [20000, 1250],
        "machine_max_acceleration_retracting": [5000, 1250],
        "machine_max_acceleration_travel": [1000, 1250],
        "machine_max_jerk_x": [9, 10], "machine_max_jerk_y": [9, 10],
        "machine_max_jerk_z": [3, 0.4]
    }))
    .unwrap();
    let mut state = MotionState::with_limits(ProcessorLimits::from_config(
        &machine,
        GCodeFlavor::Klipper,
        false,
    ));
    for line in [
        "SET_VELOCITY_LIMIT VELOCITY=500;",
        "SET_VELOCITY_LIMIT ACCEL=6000;",
        "SET_VELOCITY_LIMIT SQUARE_CORNER_VELOCITY=5;",
        "G90",
        "G28",
        "G1 X20 Y5.8 Z0.2 F18000",
    ] {
        state.motion(line);
    }
    let block = state.motion("G1 X5 Y-0.5 Z10 F18000").unwrap();
    let z_ratio = ((10.0 - f64::from(0.2_f32)) / block.distance).abs();
    assert_eq!(block.speed, 20.0 / z_ratio);
    assert_eq!(block.acceleration, 500.0 / z_ratio);
    assert_eq!(state.travel_acceleration, 6000.0);
    assert_eq!(state.max_travel_acceleration, 0.0);
    assert_eq!(&block.jerk[..2], &[5.0, 5.0]);
}

#[test]
fn emitted_axis_commands_override_configured_values() {
    let mut state = MotionState::with_limits(ProcessorLimits::from_config(
        &afinia(),
        GCodeFlavor::MarlinLegacy,
        false,
    ));
    state.motion("M201 X1000 Y1000 Z100 E10000");
    state.motion("M203 X500 Y500 Z10 E60");
    state.motion("M205 X10.00 Y10.00 Z0.30 E5.00");
    assert_eq!(state.max_acceleration, [1000.0, 1000.0, 100.0, 10000.0]);
    assert_eq!(state.max_feedrate, [500.0, 500.0, 10.0, 60.0]);
    assert_eq!(state.jerk, [10.0, 10.0, f64::from(0.3_f32), 5.0]);
    let z = state.motion("G1 Z10 F12000").unwrap();
    assert_eq!(z.speed, 10.0);
    assert_eq!(z.acceleration, 100.0);
}

#[test]
fn config_flavor_gate_uses_upstream_schema_defaults_for_unsupported_flavors() {
    let configured = afinia();
    for flavor in [
        GCodeFlavor::Repetier,
        GCodeFlavor::RepRapSprinter,
        GCodeFlavor::Teacup,
        GCodeFlavor::MakerWare,
        GCodeFlavor::Sailfish,
        GCodeFlavor::Mach3,
        GCodeFlavor::Machinekit,
        GCodeFlavor::Smoothie,
        GCodeFlavor::NoExtrusion,
    ] {
        let limits = ProcessorLimits::from_config(&configured, flavor, false);
        assert_eq!(limits.max_feedrate, [500.0, 500.0, 12.0, 120.0]);
        assert_eq!(limits.max_acceleration, [1000.0, 1000.0, 500.0, 5000.0]);
        assert_eq!(limits.jerk, [10.0, 10.0, f64::from(0.2_f32), 2.5]);
        assert_eq!(limits.print_acceleration, 1500.0);
        assert_eq!(limits.retract_acceleration, 1500.0);
    }
}

#[test]
fn only_supported_separate_travel_flavors_use_configured_travel_cap() {
    for flavor in [
        GCodeFlavor::MarlinLegacy,
        GCodeFlavor::MarlinFirmware,
        GCodeFlavor::Klipper,
        GCodeFlavor::RepRapFirmware,
    ] {
        let state =
            MotionState::with_limits(ProcessorLimits::from_config(&afinia(), flavor, false));
        let separate = matches!(
            flavor,
            GCodeFlavor::MarlinFirmware | GCodeFlavor::RepRapFirmware
        );
        assert_eq!(
            state.max_travel_acceleration,
            if separate { 20000.0 } else { 0.0 }
        );
        assert_eq!(
            state.travel_acceleration,
            if separate { 20000.0 } else { 1250.0 }
        );
        assert_eq!(state.acceleration, 20000.0);
        assert_eq!(state.retract_acceleration, 5000.0);
    }
}

#[test]
fn zero_configured_accelerations_select_upstream_initial_accelerations() {
    let machine = serde_json::from_value(serde_json::json!({
        "machine_max_acceleration_extruding": [0],
        "machine_max_acceleration_retracting": [0],
        "machine_max_acceleration_travel": [0]
    }))
    .unwrap();
    let state = MotionState::with_limits(ProcessorLimits::from_config(
        &machine,
        GCodeFlavor::MarlinFirmware,
        false,
    ));
    assert_eq!(state.acceleration, 1500.0);
    assert_eq!(state.retract_acceleration, 1500.0);
    assert_eq!(state.travel_acceleration, 1250.0);
    assert_eq!(state.max_print_acceleration, 0.0);
    assert_eq!(state.max_retract_acceleration, 0.0);
    assert_eq!(state.max_travel_acceleration, 0.0);
}

#[test]
fn emission_flag_does_not_gate_normal_mode_envelope_and_values_are_float() {
    let mut machine = afinia();
    machine.machine_max_speed_z.0[0].0 = 12.3456789;
    let enabled = ProcessorLimits::from_config(&machine, GCodeFlavor::Klipper, false);
    machine.emit_machine_limits_to_gcode.0 = false;
    let disabled = ProcessorLimits::from_config(&machine, GCodeFlavor::Klipper, false);
    assert_eq!(enabled, disabled);
    assert_eq!(disabled.max_feedrate[2], f64::from(12.3456789_f64 as f32));
    assert_eq!(disabled.jerk[2], f64::from(0.2_f32));
}

#[test]
fn junction_deviation_only_affects_marlin_firmware_not_klipper() {
    for flavor in [
        GCodeFlavor::Klipper,
        GCodeFlavor::MarlinLegacy,
        GCodeFlavor::RepRapFirmware,
        GCodeFlavor::MarlinFirmware,
    ] {
        let mut state =
            MotionState::with_limits(ProcessorLimits::from_config(&afinia(), flavor, false));
        let block = state.motion("G1 X10 F600").unwrap();
        if flavor == GCodeFlavor::MarlinFirmware {
            assert_eq!(block.jerk[2], (f64::from(0.01_f32) * 500.0 * 2.5).sqrt());
        } else {
            assert_eq!(block.jerk[2], f64::from(0.2_f32));
        }
    }
}
