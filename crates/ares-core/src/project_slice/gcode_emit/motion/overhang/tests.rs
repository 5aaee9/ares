use super::{scale_trunc, speed_for_distance};
use crate::geometry::{CoordinateScale, Line, LineDistanceTree, Point};
use crate::{FloatOrPercent, Percent};

#[test]
fn task22o50_speed_interpolation_uses_source_float_precision() {
    let sections = [
        (0.042_f32, 200.0),
        (0.105_f32, 200.0),
        (0.21_f32, 50.0),
        (0.315_f32, 30.0),
        (0.3654_f32, 10.0),
        (0.42_f32, 50.0),
    ];

    assert_eq!(speed_for_distance(0.10535, &sections, 200.0), 200.0);
}

#[test]
fn task22o137_processed_points_truncate_like_source_scaled_vectors() {
    let scale = CoordinateScale::Normal;

    assert_eq!(scale_trunc(5.006_295_746_287_667, scale), 5_006_295);
    assert_eq!(scale_trunc(-3.623_527_714_016_064_7, scale), -3_623_527);
}

/// The overhang `ref_speed` cap divides by the same fully-multiplied
/// `_mm3_per_mm` as `GCode.cpp:6660-6663` (`mm3_per_mm * print_flow_ratio *
/// filament_flow_ratio`), so a 50% band of the capped reference speed keeps
/// both flow ratios in the slowed feedrate.
#[test]
fn overhang_reference_speed_cap_divides_by_print_and_filament_flow_ratios() {
    let lines: &'static [Line] = Box::leak(
        vec![Line::new(
            Point::new(3_000_000, -1_000_000),
            Point::new(3_000_000, 1_000_000),
        )]
        .into_boxed_slice(),
    );
    let boundary = Box::leak(Box::new(LineDistanceTree::new(lines)));
    let geometry = boundary_geometry(boundary);
    let options = super::MotionOptions {
        enable_overhang_speed: true,
        slowdown_for_curled_perimeters: true,
        inner_wall_speed: 200.0,
        outer_wall_speed: 200.0,
        max_volumetric_speed: 4.0,
        print_flow_ratio: 0.5,
        filament_flow_ratio: 0.98,
        overhang_speed_bands: [
            None,
            None,
            None,
            Some(FloatOrPercent::Percent(Percent(50.0))),
        ],
        ..super::MotionOptions::default()
    };
    let properties = super::super::features::PathProperties {
        mm3_per_mm: 0.08,
        width: 0.45,
        height: 0.2,
        feature: "Inner wall",
        is_perimeter: true,
        end_clip: 0.0,
        fitting: &[],
        slope: None,
    };
    let points = [(0.0, 0.0), (1.0, 0.0)];
    let original_speed = 4.0 / (0.08 * 0.5 * 0.98);

    let processed = super::estimate(super::EstimateRequest {
        points: &points,
        properties,
        geometry,
        options: &options,
        layer_index: 1,
        original_speed,
    })
    .expect("a fully unsupported wall slows to the severe band");

    // 50% of the capped reference speed: 4 / (0.08 * 0.5 * 0.98) * 0.5.
    assert_eq!(processed[0].speed, 51.0);
}

fn boundary_geometry(
    boundary: &'static LineDistanceTree<'static>,
) -> super::LayerGeometry<'static> {
    super::LayerGeometry {
        nearest_seam_penalties: None,
        staggered_inner: false,
        internal_surfaces: &[],
        scale: CoordinateScale::Normal,
        previous_layer_boundary: Some(boundary),
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
