// Per-surface arachne wall generation tests (OrcaSlicer
// `PerimeterGenerator.cpp:2111-2176,2475-2531`).

use crate::{
    ObjectOptions, OrcaFloat, OrcaFloats, RegionOptions,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        perimeters::arachne::{
            config::validate_record,
            walls::{ArachneSpacings, InfillOverlapPercents},
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

const SCALE: CoordinateScale = CoordinateScale::Normal;

fn nozzles() -> OrcaFloats {
    OrcaFloats(vec![OrcaFloat(0.4)])
}

fn region_with(one_wall_first_layer: bool, one_wall_top: bool) -> RegionOptions {
    let mut region = RegionOptions::from_base(&crate::ProjectSettings::default().process.region);
    region.only_one_wall_first_layer.0 = one_wall_first_layer;
    region.only_one_wall_top.0 = one_wall_top;
    region
}

fn square(millimeters: i64) -> ExPolygon {
    let scaled = SCALE.checked_scale(millimeters as f64).unwrap();
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(scaled, 0),
            Point::new(scaled, scaled),
            Point::new(0, scaled),
        ]),
        Vec::new(),
    )
}

fn surface(millimeters: i64) -> RegionSurface {
    RegionSurface::new(RegionSurfaceKind::Internal, square(millimeters))
}

use crate::project_slice::perimeters::arachne::config::ArachneRecordConfig;

fn config(layer_id: usize, upper: Option<usize>) -> ArachneRecordConfig {
    config_with(layer_id, upper, region_with(false, false))
}

fn config_with(
    layer_id: usize,
    upper: Option<usize>,
    region: RegionOptions,
) -> ArachneRecordConfig {
    validate_record(
        &region,
        &ObjectOptions::from_base(&crate::ProjectSettings::default().process.object),
        layer_id,
        upper,
        false,
        0.012,
        &nozzles(),
        SCALE,
    )
    .unwrap()
}

fn spacings() -> ArachneSpacings {
    ArachneSpacings::new(SCALE, 0.385, 0.42, 0.39, 0.42).unwrap()
}

#[test]
fn task22w12_generates_real_variable_width_walls_for_one_island() {
    let config = config(1, Some(2));
    let generated = crate::project_slice::perimeters::arachne::walls::generate_surface_walls(
        &surface(20),
        &config,
        &spacings(),
        3,
        0.16,
        SCALE,
    )
    .unwrap();

    // Three wall levels for `wall_loops = 3` with a filled inner contour
    // (`PerimeterGenerator.cpp:2148-2176`).
    assert_eq!(generated.toolpaths.len(), 3);
    let lines = generated.toolpaths.iter().flatten().count();
    assert!(lines > 0);
    assert!(!generated.infill_contour.is_empty());
}

#[test]
fn task22w13_bottom_layer_single_wall_rule_resets_the_loop_count() {
    // `only_one_wall_first_layer` with `raft_layers = 0`
    // (`PerimeterGenerator.cpp:2121-2122`).
    let config = config_with(0, Some(1), region_with(true, false));
    let generated = crate::project_slice::perimeters::arachne::walls::generate_surface_walls(
        &surface(20),
        &config,
        &spacings(),
        3,
        0.16,
        SCALE,
    )
    .unwrap();

    assert_eq!(generated.toolpaths.len(), 1);
}

#[test]
fn task22w14_topmost_layer_single_wall_rule_resets_the_loop_count() {
    // `only_one_wall_top` on the topmost layer
    // (`PerimeterGenerator.cpp:2125-2127`).
    let config = config_with(11, None, region_with(false, true));
    let generated = crate::project_slice::perimeters::arachne::walls::generate_surface_walls(
        &surface(20),
        &config,
        &spacings(),
        3,
        0.16,
        SCALE,
    )
    .unwrap();

    assert!(config.is_topmost_layer);
    assert_eq!(generated.toolpaths.len(), 1);
}

#[test]
fn task22w15_ext_perimeter_spacing2_averages_the_plain_spacings() {
    let spacings = spacings();
    // `PerimeterGenerator.cpp:2103`: 0.5 * (0.39f32 + 0.385f32) truncates
    // through the source f32 sum and `scaled<coord_t>` (`Point.hpp:662-668`).
    assert_eq!(spacings.ext_perimeter_spacing2, 387_499);
}

#[test]
fn task22w16_infill_boundary_applies_the_wall_overlap_percent() {
    let generated = crate::project_slice::perimeters::arachne::walls::generate_surface_walls(
        &surface(20),
        &config(1, Some(2)),
        &spacings(),
        3,
        0.16,
        SCALE,
    )
    .unwrap();
    let boundary = crate::project_slice::perimeters::arachne::walls::surface_infill_boundary(
        generated.infill_contour,
        generated.toolpaths.len(),
        generated.toolpaths.len() as i32 - 1,
        &spacings(),
        0.012,
        InfillOverlapPercents {
            infill_wall_overlap: 20.0,
            top_bottom_infill_wall_overlap: 25.0,
        },
        false,
    )
    .unwrap();

    // Three walls keep non-empty infill surfaces; the no-overlap set stays
    // inside the overlap-expanded fill surfaces.
    assert!(!boundary.fill_surfaces.is_empty());
    assert!(!boundary.fill_no_overlap.is_empty());
}
