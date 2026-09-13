use super::*;
use crate::geometry::CoordinateScale;
use crate::project_slice::perimeters::classic::materialize::Polyline3;

#[test]
fn wipe_before_external_uses_the_source_rotated_inner_point() {
    let points = [
        (4_690_000, 4_690_000),
        (-4_690_000, 4_690_000),
        (-4_690_000, -4_690_000),
        (4_690_000, -4_690_000),
        (4_690_000, 4_690_000),
    ]
    .into_iter()
    .map(|(x, y)| Point3 { x, y, z: 200_000 })
    .collect();
    let paths = vec![ExtrusionPath {
        polyline: Polyline3 {
            points,
            fitting: Vec::new(),
        },
        role: ExtrusionRole::ExternalPerimeter,
        can_reverse: true,
        mm3_per_mm: 0.04,
        width: 0.42,
        height: 0.2,
    }];
    let mut state = EmitState {
        offset: (110.0, 110.0),
        origin: (110.0, 110.0),
        travel_feedrate: 9_000.0,
        options: super::super::MotionOptions {
            wipe_before_external_loop: true,
            wall_loops: 2,
            nozzle_diameter: 0.4,
            seam_gap: 0.04,
            ..super::super::MotionOptions::default()
        },
        ..EmitState::default()
    };
    let mut output = Vec::new();

    append_wipe_before_external(
        &mut output,
        &paths,
        ExtrusionLoopRole::Default,
        LayerGeometry {
            nearest_seam_penalties: None,
            staggered_inner: false,
            internal_surfaces: &[],
            scale: CoordinateScale::Normal,
            previous_layer_boundary: None,
            avoid_crossing: super::super::state::AvoidCrossingGeometry {
                external_perimeter_width: 0.42,
                layer_slices: &[],
                perimeter_spacing: 0.0,
                top_surfaces: &[],
                chunk_slices: &[],
                chunk_perimeter_spacing: 0.0,
            },
        },
        &mut state,
    );

    assert_eq!(output, b"G1 X114.49 Y114.344 F9000\n");
    assert_eq!(
        state.pending_wipe_before_external_target,
        Some(super::super::arc::Point {
            x: 114.69,
            y: 114.69,
        })
    );
}
