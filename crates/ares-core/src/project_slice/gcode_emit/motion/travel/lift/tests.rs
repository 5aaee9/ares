use super::super::EmitState;
use super::append_eager;
use crate::project_slice::gcode_emit::motion::MotionOptions;

/// `process_layer` maps Auto, Spiral AND Slope hop types onto SpiralLift for
/// the layer-chunk retract (`GCode.cpp:4657-4660`); the BBL eager layer-start
/// lift therefore takes the spiral form for all three. Regression: with
/// `z_hop_types = Slope Lift`, the eager lift must emit the `G17/G3 … P1`
/// arc (radius = z_hop / (2π·atan(travel_slope))) like the OrcaSlicer oracle,
/// not a plain `G1 Z`.
#[test]
fn eager_layer_start_lift_takes_the_spiral_form_for_slope_type() {
    let mut state = EmitState {
        options: MotionOptions {
            z_hop: 0.4,
            z_hop_type: crate::ZHopType::Slope,
            travel_slope_radians: 3.0_f64.to_radians(),
            travel_feedrate: 30_000.0,
            enable_arc_fitting: true,
            ..MotionOptions::default()
        },
        layer_z: 1.2,
        positioned: true,
        ..EmitState::default()
    };
    let mut output = Vec::new();

    append_eager(&mut output, &mut state);

    // radius = 0.4 / (2π·atan(3°)) ≈ 1.2169 → I1.217 with 3-digit offsets.
    let text = String::from_utf8(output).unwrap();
    assert!(
        text.contains("G17\n"),
        "expected the arc plane selection, got {text:?}"
    );
    assert!(
        text.contains("G3 Z1.6 I1.217 J0 P1  F0\n"),
        "expected the spiral-lift arc, got {text:?}"
    );
    assert!(
        !text.contains("\nG1 Z1.6 F"),
        "must not fall back to the plain lift"
    );
}
