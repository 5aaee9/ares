//! `CoolingBuffer.cpp:865–869`: internal-bridge END always requests fan emission,
//! including when an equal-speed overhang START is suppressed (:780, :851).
use super::*;

#[test]
fn cooling_buffer_internal_bridge_end_forces_equal_speed_outer_wall_fan() {
    let mut state = EmitState {
        options: MotionOptions {
            enable_overhang_bridge_fan: true,
            overhang_fan_speed: 100,
            overhang_fan_threshold: crate::RawOverhangFanThreshold::Percent0,
            ..MotionOptions::default()
        },
        layer_index: 63,
        part_fan_speed: 100,
        physical_fan_speed: 100,
        ..EmitState::default()
    };
    let mut output = Vec::new();
    update_for_constant_path(&mut output, properties("Internal Bridge"), &mut state);
    output.clear();
    state.layer_index = 64;
    update_for_constant_path(&mut output, properties("Outer wall"), &mut state);
    assert_eq!(
        output,
        b";__ARES_ROLE_FAN_CONDITIONAL_100__\n;__ARES_ROLE_FAN_CONDITIONAL_FORCE_100__\n"
    );
}
