#[tokio::test]
async fn dynamic_outer_wall_travels_to_the_unprocessed_first_point() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let wipe_end = lines
        .windows(4)
        .position(|lines| {
            lines[0] == "; WIPE_END" && lines[1] == "G17" && lines[3] == "G1 X133.539 Y89.629 Z1.2"
        })
        .unwrap();

    assert_eq!(lines[wipe_end + 2], "G3 Z1.2 I-.045 J1.216 P1  F60000");
    assert_eq!(lines[wipe_end + 3], "G1 X133.539 Y89.629 Z1.2");
}

use super::{MotionOptions, begin_layer, path};
use crate::geometry::CoordinateScale;
use crate::project_slice::gcode_emit::motion::{
    EmitState, LayerGeometry, features::PathProperties, state::AvoidCrossingGeometry,
};

/// `GCode.cpp:6469-6471` builds `_mm3_per_mm = path.mm3_per_mm *
/// print_flow_ratio * filament_flow_ratio`, and the volumetric speed cap
/// (`GCode.cpp:6615-6617`) divides `filament_max_volumetric_speed` by that
/// fully-multiplied value, so both flow ratios cap the speed.
#[test]
fn volumetric_speed_cap_divides_by_print_and_filament_flow_ratios() {
    let mut state = EmitState {
        options: MotionOptions {
            inner_wall_speed: 200.0,
            max_volumetric_speed: 4.0,
            print_flow_ratio: 0.5,
            filament_flow_ratio: 0.98,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    begin_layer(&mut Vec::new(), &mut state, 1, 0.4, 0.2);
    let mut output = Vec::new();

    path::emit(
        &mut output,
        [(0, 0), (1_000_000, 0)].into_iter(),
        PathProperties {
            mm3_per_mm: 0.08,
            width: 0.45,
            height: 0.2,
            feature: "Inner wall",
            is_perimeter: true,
            end_clip: 0.0,
            fitting: &[],
            slope: None,
        },
        test_geometry(),
        &mut state,
    );

    let expected = 4.0 / (0.08 * 0.5 * 0.98) * 60.0;
    assert!(
        (state.extrusion_feedrate - expected).abs() < 1e-9,
        "feedrate {} vs upstream _mm3_per_mm cap {}",
        state.extrusion_feedrate,
        expected
    );
    let output = String::from_utf8(output).unwrap();
    assert!(
        output.contains("G1 F6122.449;_EXTRUDE_SET_SPEED"),
        "speed line must carry the capped feedrate: {output}"
    );
}

/// `GCode.cpp:6615` only applies the volumetric cap while
/// `filament_max_volumetric_speed > 0`; zero leaves the role speed alone.
#[test]
fn zero_max_volumetric_speed_leaves_the_role_speed_uncapped() {
    let mut state = EmitState {
        options: MotionOptions {
            inner_wall_speed: 200.0,
            max_volumetric_speed: 0.0,
            print_flow_ratio: 0.5,
            filament_flow_ratio: 0.98,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    begin_layer(&mut Vec::new(), &mut state, 1, 0.4, 0.2);
    let mut output = Vec::new();

    path::emit(
        &mut output,
        [(0, 0), (1_000_000, 0)].into_iter(),
        PathProperties {
            mm3_per_mm: 0.08,
            width: 0.45,
            height: 0.2,
            feature: "Inner wall",
            is_perimeter: true,
            end_clip: 0.0,
            fitting: &[],
            slope: None,
        },
        test_geometry(),
        &mut state,
    );

    assert!(
        (state.extrusion_feedrate - 200.0 * 60.0).abs() < 1e-9,
        "feedrate {} must keep the uncapped role speed",
        state.extrusion_feedrate
    );
}

fn test_geometry() -> LayerGeometry<'static> {
    LayerGeometry {
        nearest_seam_penalties: None,
        staggered_inner: false,
        internal_surfaces: &[],
        scale: CoordinateScale::Normal,
        previous_layer_boundary: None,
        avoid_crossing: AvoidCrossingGeometry {
            external_perimeter_width: 0.42,
            layer_slices: &[],
            perimeter_spacing: 0.0,
            top_surfaces: &[],
        },
    }
}
