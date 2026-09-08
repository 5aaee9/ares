mod role_end;

use super::{ProcessedPoint, update_for_constant_path, update_for_variable_segment};
use crate::project_slice::gcode_emit::motion::{
    EmitState, MotionOptions, features::PathProperties,
};

fn properties(feature: &'static str) -> PathProperties<'static> {
    PathProperties {
        mm3_per_mm: 0.04,
        width: 0.45,
        height: 0.2,
        feature,
        is_perimeter: true,
        end_clip: 0.0,
        fitting: &[],
        slope: None,
    }
}

#[test]
fn equal_speed_ignores_start_and_reasserts_base_fan_at_end() {
    let mut state = EmitState {
        options: MotionOptions {
            enable_overhang_bridge_fan: true,
            overhang_fan_speed: 100,
            ..MotionOptions::default()
        },
        layer_index: 13,
        part_fan_speed: 100,
        physical_fan_speed: 100,
        ..EmitState::default()
    };
    let mut output = Vec::new();

    update_for_constant_path(&mut output, properties("Overhang wall"), &mut state);
    update_for_variable_segment(
        &mut output,
        properties("Inner wall"),
        ProcessedPoint {
            x: 0.0,
            y: 0.0,
            speed: 50.0,
            overlap: 1.0,
        },
        ProcessedPoint {
            x: 1.0,
            y: 0.0,
            speed: 50.0,
            overlap: 1.0,
        },
        &mut state,
    );

    assert_eq!(
        output,
        b";__ARES_ROLE_FAN_CONDITIONAL_100__\n;__ARES_ROLE_FAN_BASE__\n"
    );
    assert!(!state.overhang_fan_active);
}

#[test]
fn closed_layer_reasserts_baseline_when_overhang_region_ends() {
    let mut state = EmitState {
        options: MotionOptions {
            enable_overhang_bridge_fan: true,
            overhang_fan_speed: 100,
            overhang_fan_threshold: crate::RawOverhangFanThreshold::Percent0,
            close_fan_first_layers: 1,
            ..MotionOptions::default()
        },
        layer_index: 0,
        ..EmitState::default()
    };
    let mut output = Vec::new();

    update_for_constant_path(&mut output, properties("Outer wall"), &mut state);
    update_for_constant_path(&mut output, properties("Bottom surface"), &mut state);

    assert_eq!(output, b";__ARES_ROLE_FAN_BASE__\n");
}

#[test]
fn faster_overhang_fan_activates_for_variable_segment() {
    let mut state = EmitState {
        options: MotionOptions {
            enable_overhang_bridge_fan: true,
            overhang_fan_speed: 100,
            ..MotionOptions::default()
        },
        layer_index: 3,
        part_fan_speed: 40,
        physical_fan_speed: 40,
        ..EmitState::default()
    };
    let point = ProcessedPoint {
        x: 0.0,
        y: 0.0,
        speed: 50.0,
        overlap: 0.0,
    };
    let mut output = Vec::new();

    update_for_variable_segment(
        &mut output,
        properties("Inner wall"),
        point,
        point,
        &mut state,
    );

    assert_eq!(output, b";__ARES_ROLE_FAN_CONDITIONAL_100__\n");
    assert!(state.overhang_fan_active);
}

#[test]
fn internal_bridge_marker_reasserts_equal_baseline_at_end() {
    let mut state = EmitState {
        options: MotionOptions {
            enable_overhang_bridge_fan: true,
            overhang_fan_speed: 100,
            overhang_fan_threshold: crate::RawOverhangFanThreshold::Percent50,
            ..MotionOptions::default()
        },
        layer_index: 13,
        part_fan_speed: 100,
        physical_fan_speed: 100,
        ..EmitState::default()
    };
    let point = ProcessedPoint {
        x: 0.0,
        y: 0.0,
        speed: 50.0,
        overlap: 1.0,
    };
    let mut output = Vec::new();

    update_for_variable_segment(
        &mut output,
        properties("Internal Bridge"),
        point,
        point,
        &mut state,
    );
    update_for_constant_path(&mut output, properties("Inner wall"), &mut state);

    assert_eq!(
        output,
        b";__ARES_ROLE_FAN_CONDITIONAL_100__\n;__ARES_ROLE_FAN_BASE__\n"
    );
}

#[test]
fn explicit_internal_bridge_speed_overrides_and_restores_baseline() {
    let mut state = EmitState {
        options: MotionOptions {
            enable_overhang_bridge_fan: true,
            internal_bridge_fan_speed: crate::options::InternalBridgeFanSpeed::new(75),
            ..MotionOptions::default()
        },
        layer_index: 13,
        part_fan_speed: 40,
        physical_fan_speed: 40,
        ..EmitState::default()
    };
    let mut output = Vec::new();

    update_for_constant_path(&mut output, properties("Internal Bridge"), &mut state);
    update_for_constant_path(&mut output, properties("Inner wall"), &mut state);

    assert_eq!(
        output,
        b";__ARES_ROLE_FAN_FIXED_75__\n;__ARES_ROLE_FAN_BASE__\n"
    );
}
