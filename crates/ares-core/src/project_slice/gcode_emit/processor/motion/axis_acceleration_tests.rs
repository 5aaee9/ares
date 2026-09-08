//! `GCodeProcessor.cpp:4066–4073`: moving-axis limits include zero and fractions.
use super::{MotionState, arc};
use crate::project_slice::gcode_emit::processor::ProcessorLimits;
use crate::{GCodeFlavor, MachineEnvelopeOptions};

fn state(z_limits: &[f64]) -> MotionState {
    let machine: MachineEnvelopeOptions = serde_json::from_value(serde_json::json!({
        "machine_max_acceleration_z": z_limits
    }))
    .unwrap();
    MotionState::with_limits(ProcessorLimits::from_config(
        &machine,
        GCodeFlavor::Klipper,
        false,
    ))
}

#[test]
fn zero_axis_acceleration_is_preserved_in_linear_block() {
    let block = state(&[0.0]).motion("G1 Z10 F600").unwrap();
    assert_eq!(block.acceleration, 0.0);
}

#[test]
fn sub_unit_axis_acceleration_is_preserved_in_linear_block() {
    let block = state(&[0.5]).motion("G1 Z10 F600").unwrap();
    assert_eq!(block.acceleration, 0.5);
}

#[test]
fn zero_axis_acceleration_is_preserved_in_arc_segments() {
    let blocks = state(&[0.0]).motions("G3 X10 Y10 Z100 I0 J10 F600");
    assert!(blocks.len() > 1);
    for block in blocks {
        assert!(block.direction[2] > 0.0);
        assert_eq!(block.acceleration, 0.0);
    }
}

#[test]
fn sub_unit_axis_acceleration_is_preserved_in_arc_segments() {
    let code = "G3 X10 Y10 Z100 I0 J10 F600";
    let blocks = state(&[0.5]).motions(code);
    let deltas = arc::deltas(
        "G3",
        code,
        arc::ArcMotion {
            start: [0.0; 3],
            end: [10.0, 10.0, 100.0],
            e_delta: 0.0,
            feedrate: 10.0,
            gcode_flavor: GCodeFlavor::Klipper,
        },
    )
    .unwrap();
    assert!(blocks.len() > 1);
    assert_eq!(blocks.len(), deltas.len());
    for (block, delta) in blocks.into_iter().zip(deltas) {
        // GCodeProcessor.cpp:4069–4071 uses displacement and inverse distance.
        // Use source deltas, not the separately rounded direction multiplication.
        let expected = 0.5 / (delta[2] / block.distance).abs();
        assert!(expected > 0.5 && expected < 1.0);
        assert_eq!(block.acceleration, expected);
    }
}

#[test]
fn empty_axis_acceleration_array_clamps_linear_and_arc_blocks_to_zero() {
    // `get_option_value` returns zero for an empty ConfigOptionFloats array.
    let mut linear = state(&[]);
    assert_eq!(linear.max_acceleration[2], 0.0);
    assert_eq!(linear.motion("G1 Z10 F600").unwrap().acceleration, 0.0);
    let blocks = state(&[]).motions("G3 X10 Y10 Z100 I0 J10 F600");
    assert!(blocks.len() > 1);
    for block in blocks {
        assert_eq!(block.acceleration, 0.0);
    }
}
