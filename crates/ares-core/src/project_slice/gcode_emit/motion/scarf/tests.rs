use super::build;
use crate::geometry::CoordinateScale;
use crate::project_slice::gcode_emit::motion::{
    EmitState, LayerGeometry, MotionOptions, features::PathProperties, options::ScarfOptions,
};
use crate::project_slice::perimeters::classic::{
    chained_loops::ExtrusionLoopRole,
    materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
};
use crate::{FloatOrPercent, Percent, ProcessSeamScarfType};

fn square() -> Vec<ExtrusionPath> {
    vec![ExtrusionPath {
        polyline: Polyline3 {
            points: [
                (0, 0),
                (10_000_000, 0),
                (10_000_000, 10_000_000),
                (0, 10_000_000),
                (0, 0),
            ]
            .map(|(x, y)| Point3 { x, y, z: 0 })
            .to_vec(),
            fitting: Vec::new(),
        },
        role: ExtrusionRole::ExternalPerimeter,
        can_reverse: false,
        mm3_per_mm: 0.08,
        width: 0.45,
        height: 0.2,
    }]
}

fn geometry() -> LayerGeometry<'static> {
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
    }
}

fn options(kind: ProcessSeamScarfType) -> MotionOptions {
    MotionOptions {
        outer_wall_speed: 50.0,
        inner_wall_speed: 80.0,
        small_perimeter_speed: 25.0,
        seam_gap: 0.1,
        scarf: ScarfOptions {
            seam_slope_type: kind,
            start_height: Some(FloatOrPercent::Float(0.0)),
            min_length: 20.0,
            steps: 10,
            speed: Some(FloatOrPercent::Percent(Percent(100.0))),
            flow_ratio: 1.0,
            ..ScarfOptions::default()
        },
        ..MotionOptions::default()
    }
}

#[test]
fn source_enum_gate_distinguishes_contours_and_holes_after_first_layer() {
    let paths = square();
    assert!(
        build(
            &paths,
            ExtrusionLoopRole::Default,
            geometry(),
            &options(ProcessSeamScarfType::None),
            1,
        )
        .is_none()
    );
    assert!(
        build(
            &paths,
            ExtrusionLoopRole::Default,
            geometry(),
            &options(ProcessSeamScarfType::External),
            1,
        )
        .is_some()
    );
    assert!(
        build(
            &paths,
            ExtrusionLoopRole::Hole,
            geometry(),
            &options(ProcessSeamScarfType::External),
            1,
        )
        .is_none()
    );
    assert!(
        build(
            &paths,
            ExtrusionLoopRole::Hole,
            geometry(),
            &options(ProcessSeamScarfType::All),
            1,
        )
        .is_some()
    );
}

#[test]
fn source_scarf_recursively_segments_and_clips_both_overlaps() {
    let scarf = build(
        &square(),
        ExtrusionLoopRole::Default,
        geometry(),
        &options(ProcessSeamScarfType::External),
        1,
    )
    .unwrap();

    assert_eq!(scarf.paths.len(), 3);
    assert_eq!(scarf.paths[0].path.polyline.points.len(), 17);
    assert_eq!(scarf.paths[0].path.polyline.points[0].x, 200_000);
    assert_eq!(scarf.paths[1].path.polyline.points.len(), 3);
    assert_eq!(scarf.paths[2].path.polyline.points.len(), 17);
    assert_eq!(
        scarf.paths[2].path.polyline.points.last().unwrap().y,
        9_799_999
    );
    assert_eq!(scarf.paths[0].slope.unwrap().e_begin, 0.0);
    assert_eq!(scarf.paths[2].slope.unwrap().z_begin, 1.0);
}

#[test]
fn sloped_segments_interpolate_z_and_endpoint_extrusion_ratio() {
    let mut state = EmitState {
        layer_z: 0.4,
        options: MotionOptions {
            filament_area: 1.0,
            filament_flow_ratio: 1.0,
            print_flow_ratio: 1.0,
            use_relative_e_distances: true,
            ..MotionOptions::default()
        },
        ..EmitState::default()
    };
    let properties = PathProperties {
        mm3_per_mm: 1.0,
        width: 0.45,
        height: 0.2,
        feature: "Outer wall",
        is_perimeter: true,
        end_clip: 0.0,
        fitting: &[],
        slope: None,
    };
    let mut output = Vec::new();

    super::emit_segments(
        &mut output,
        &[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)],
        super::Slope {
            z_begin: 0.0,
            z_end: 1.0,
            e_begin: 0.0,
            e_end: 1.0,
            speed: 50.0,
            flow_ratio: 1.0,
        },
        properties,
        &mut state,
    );

    assert_eq!(output, b"G1 X1 Y0 Z.3 E.5\nG1 X2 Y0 Z.4 E1\n");
}
