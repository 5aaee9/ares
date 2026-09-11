use crate::{
    ObjectOptions, OrcaFloat, OrcaFloats, Percent, RegionOptions, SliceError,
    geometry::CoordinateScale, project_slice::perimeters::arachne::config::validate_record,
};

const SCALE: CoordinateScale = CoordinateScale::Normal;

fn nozzles() -> OrcaFloats {
    OrcaFloats(vec![OrcaFloat(0.4)])
}

fn object() -> ObjectOptions {
    ObjectOptions::from_base(&crate::ProjectSettings::default().process.object)
}

fn default_region() -> RegionOptions {
    RegionOptions::from_base(&crate::ProjectSettings::default().process.region)
}

fn validate(
    region: &RegionOptions,
    layer_id: usize,
    upper: Option<usize>,
) -> Result<crate::project_slice::perimeters::arachne::config::ArachneRecordConfig, SliceError> {
    validate_record(
        region,
        &object(),
        layer_id,
        upper,
        false,
        0.012,
        &nozzles(),
        SCALE,
    )
}

#[test]
fn task22w6_make_paths_params_scales_percent_options_by_min_nozzle() {
    // Snapmaker A250 defaults: min_feature_size 25%, min_bead_width 85%,
    // wall_transition_length 100%, filter deviation 25% of the 0.4mm
    // nozzle (`WallToolPaths.cpp:31-46`).
    let config = validate(&default_region(), 3, Some(4)).unwrap();
    assert_eq!(config.params.min_feature_size, 100_000);
    assert_eq!(config.params.min_bead_width, 340_000);
    assert_eq!(config.params.wall_transition_length, 400_000);
    assert_eq!(config.params.wall_transition_filter_deviation, 100_000);
    assert_eq!(config.params.wall_transition_angle, 10.0);
    assert_eq!(config.params.wall_distribution_count, 1);
    assert_eq!(config.params.wall_maximum_resolution, 500_000);
    assert_eq!(config.params.wall_maximum_deviation, 25_000);
    assert_eq!(config.params.min_length_factor, 0.5);
}

#[test]
fn task22w7_layer_zero_uses_the_initial_layer_min_bead_width() {
    let config = validate(&default_region(), 0, Some(1)).unwrap();
    // `initial_layer_min_bead_width` 85% (`WallToolPaths.cpp:40-42`).
    assert_eq!(config.params.min_bead_width, 340_000);
    assert!(config.is_bottom_layer);
    assert!(!config.is_topmost_layer);
}

#[test]
fn task22w8_arc_fitting_reduces_the_surface_simplify_resolution() {
    let region = default_region();
    let base = validate(&region, 3, Some(4)).unwrap();
    let arc = validate_record(
        &region,
        &object(),
        3,
        Some(4),
        true,
        0.012,
        &nozzles(),
        SCALE,
    )
    .unwrap();
    // `PerimeterGenerator.cpp:2119-2122`: the resolution is scaled.
    assert_eq!(base.surface_simplify_resolution, 12000.0);
    assert_eq!(arc.surface_simplify_resolution, 0.2 * 12000.0);
}

#[test]
fn task22w9_rejects_deferred_scopes_typed() {
    let mut region = default_region();
    region.overhang_reverse.0 = true;
    assert_eq!(
        validate(&region, 3, Some(4)).unwrap_err(),
        SliceError::UnsupportedProjectFeature("overhang_reverse".to_owned())
    );
    // wall_sequence InnerOuterInner is now implemented by the sandwich
    // reordering (`PerimeterGenerator.cpp:2374-2464`); only the scope
    // gates below remain typed rejections.
    let mut region = default_region();
    region.only_one_wall_top.0 = true;
    assert_eq!(
        validate(&region, 3, Some(4)).unwrap_err(),
        SliceError::UnsupportedProjectFeature("only_one_wall_top".to_owned())
    );
    let mut region = default_region();
    region.alternate_extra_wall.0 = true;
    region.sparse_infill_density = Percent(15.0);
    assert_eq!(
        validate(&region, 1, Some(2)).unwrap_err(),
        SliceError::UnsupportedProjectFeature("alternate_extra_wall".to_owned())
    );
}

#[test]
fn task22w10_inner_outer_sequence_keeps_outer_wall_last() {
    // Snapmaker default `wall_sequence = inner wall/outer wall`
    // (`PerimeterGenerator.cpp:2273-2280`).
    let region = default_region();
    assert!(!validate(&region, 3, Some(4)).unwrap().outer_wall_first);
    assert!(!validate(&region, 0, Some(1)).unwrap().outer_wall_first);
    let mut region = default_region();
    region.wall_sequence = crate::ProcessWallSequence::OuterInner;
    assert!(validate(&region, 3, Some(4)).unwrap().outer_wall_first);
}

#[test]
fn task22w11_precise_outer_wall_requires_the_inner_outer_sequence() {
    let region = default_region();
    let config = validate(&region, 3, Some(4)).unwrap();
    assert!(config.precise_outer_wall);
    let mut region = default_region();
    region.wall_sequence = crate::ProcessWallSequence::OuterInner;
    assert!(!validate(&region, 3, Some(4)).unwrap().precise_outer_wall);
}
