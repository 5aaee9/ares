//! `CoolingBuffer.cpp:780,856–869,984–1006`: END emission survives an inactive
//! conditional START; the final layer baseline still selects the fan speed.
use super::super::*;

#[test]
fn cooling_buffer_role_end_emits_even_when_conditional_speed_does_not_exceed_baseline() {
    let mut cooling = CoolingState {
        part_speed: 100,
        physical_part_speed: 100,
        provisional_part_speed: 100,
        pending_layer_index: None,
        part_fan_ramp: PartCoolingFanRamp::new(PartCoolingFanRampConfig {
            min_speed: 100.0,
            max_speed: 100.0,
            full_speed_layer: 0,
            close_fan_first_layers: 0,
            layer_times_s: [0.0, 0.0],
            fan_kickstart_s: 0.0,
            reduce_fan_stop_start_freq: false,
        }),
        additional_speed: 0,
        additional_fan_speed: 0,
        auxiliary_fan: false,
        part_cooling_fan_min_pwm: 0,
        emit_initial_fan: false,
        fan_mover_enabled: false,
        feedrate: feedrate::State::new(
            feedrate::Config {
                enabled: false,
                target_time: 0.0,
                minimum_speed: 0.0,
                keep_outer_wall_speed: false,
                relative_e: true,
            },
            120.0,
        ),
    };
    for (speed, baseline, force, expected) in [
        (100, 100, false, ""),
        (100, 100, true, "M106 S255\n"),
        (75, 100, true, "M106 S255\n"),
        (100, 40, true, "M106 S255\n"),
        (100, 40, false, "M106 S255\n"),
    ] {
        cooling.physical_part_speed = baseline;
        let mut output = Vec::new();
        append_deferred_role_fan(&mut output, DeferredRoleFan::Conditional { speed, force });
        cooling.resolve_role_fans(&mut output, 0, baseline);
        assert_eq!(
            output,
            expected.as_bytes(),
            "speed={speed}, baseline={baseline}, force={force}"
        );
    }
}
