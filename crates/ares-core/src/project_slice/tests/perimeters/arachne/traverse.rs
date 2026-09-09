// Arachne integration seam 3 tests: the `traverse_extrusions` port
// (OrcaSlicer `PerimeterGenerator.cpp:370-574`) converting ordered arachne
// wall lines into typed `ExtrusionEntity` vocabulary — closed loops with
// source loop roles and orientation, open multi-paths, steep-overhang flags,
// and the fail-closed deferred branches.

use crate::{
    OrcaBool, OrcaFloat, OrcaInt, Percent, ProcessFuzzySkinMode, ProcessFuzzySkinType,
    ProcessNoiseType, ProcessRegionSourceOptions, ProcessWallDirection, ProcessWallSequence,
    RegionOptions, SliceError,
    arachne::{ExtrusionJunction, ExtrusionLine},
    geometry::{CoordinateScale, Point},
    perimeters::FuzzySkinConfig,
    project_slice::perimeters::{
        arachne::{
            PerimeterGeneratorArachneExtrusion, TraverseExtrusionsContext, traverse_extrusions,
        },
        classic::{
            chained_loops::ExtrusionLoopRole, entity_collections::ExtrusionEntity,
            materialize::ExtrusionRole,
        },
        types::Flow,
    },
};

const SCALE: CoordinateScale = CoordinateScale::Normal;
const ROUNDED_RECTANGLE_FACTOR: f32 = (1.0 - 0.25 * std::f64::consts::PI) as f32;

fn flow(height: f32) -> Flow {
    Flow {
        width: 0.4,
        height,
        spacing: 0.35,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 0.08,
    }
}

fn context() -> TraverseExtrusionsContext {
    TraverseExtrusionsContext {
        layer_id: 2,
        raft_layers: 0,
        detect_overhang_wall: false,
        overhang_reverse: false,
        wall_direction: ProcessWallDirection::CounterClockwise,
        wall_sequence: ProcessWallSequence::InnerOuter,
        fuzzy_skin: FuzzySkinConfig::disabled(),
        perimeter_flow: flow(0.3),
        ext_perimeter_flow: flow(0.2),
        scale: SCALE,
    }
}

fn line(inset_index: usize, is_closed: bool, junctions: &[(i64, i64, i64)]) -> ExtrusionLine {
    let mut line = ExtrusionLine::new(inset_index, false);
    line.is_closed = is_closed;
    for &(x, y, width) in junctions {
        line.push(ExtrusionJunction::new(Point::new(x, y), width, inset_index));
    }
    line
}

fn wall(
    inset_index: usize,
    is_closed: bool,
    is_contour: bool,
    junctions: &[(i64, i64, i64)],
) -> PerimeterGeneratorArachneExtrusion {
    PerimeterGeneratorArachneExtrusion {
        extrusion: line(inset_index, is_closed, junctions),
        is_contour,
    }
}

fn rectangle(width: i64) -> Vec<(i64, i64, i64)> {
    vec![
        (0, 0, width),
        (1_000_000, 0, width),
        (1_000_000, 1_000_000, width),
        (0, 1_000_000, width),
        (0, 0, width),
    ]
}

fn active_fuzzy_skin() -> FuzzySkinConfig {
    let mut region = RegionOptions::from_base(&ProcessRegionSourceOptions::default());
    region.fuzzy_skin = ProcessFuzzySkinType::External;
    region.fuzzy_skin_first_layer = OrcaBool(true);
    region.fuzzy_skin_noise_type = ProcessNoiseType::Ripple;
    region.fuzzy_skin_mode = ProcessFuzzySkinMode::Displacement;
    region.fuzzy_skin_thickness = OrcaFloat(0.2);
    region.fuzzy_skin_point_distance = OrcaFloat(1.0);
    region.fuzzy_skin_ripples_per_layer = OrcaInt(1);
    region.fuzzy_skin_ripple_offset = Percent(0.0);
    region.fuzzy_skin_layers_between_ripple_offset = OrcaInt(1);
    FuzzySkinConfig::from_region(&region)
}

#[test]
fn closed_contour_line_becomes_a_counter_clockwise_loop_entity() {
    let outcome = traverse_extrusions(vec![wall(0, true, true, &rectangle(450_000))], &context())
        .expect("the ordinary branch converts a closed contour");
    let [ExtrusionEntity::Loop(ordered)] = &outcome.collection.entities[..] else {
        panic!("a closed wall line appends one loop entity");
    };
    assert_eq!(ordered.extrusion_loop.role, ExtrusionLoopRole::Default);
    assert_eq!(ordered.inset_idx, -1);
    let [path] = &ordered.extrusion_loop.paths[..] else {
        panic!("a uniform-width loop keeps one sub-path");
    };
    assert_eq!(path.role, ExtrusionRole::ExternalPerimeter);
    assert_eq!(path.height, 0.2);
    let expected_width = 450_000_f32 * SCALE.factor() as f32 + 0.2 * ROUNDED_RECTANGLE_FACTOR;
    assert_eq!(path.width, expected_width);
    // `make_counter_clockwise` keeps the source counter-clockwise order.
    let points: Vec<(i64, i64)> = path
        .polyline
        .points
        .iter()
        .map(|point| (point.x, point.y))
        .collect();
    assert_eq!(
        points,
        vec![
            (0, 0),
            (1_000_000, 0),
            (1_000_000, 1_000_000),
            (0, 1_000_000),
            (0, 0),
        ]
    );
    assert!(!outcome.steep_overhang_contour && !outcome.steep_overhang_hole);
}

#[test]
fn closed_hole_line_reverses_to_clockwise_and_keeps_hole_role() {
    let outcome = traverse_extrusions(vec![wall(0, true, false, &rectangle(450_000))], &context())
        .expect("the ordinary branch converts a closed hole");
    let [ExtrusionEntity::Loop(ordered)] = &outcome.collection.entities[..] else {
        panic!("a closed wall line appends one loop entity");
    };
    assert_eq!(ordered.extrusion_loop.role, ExtrusionLoopRole::Hole);
    let [path] = &ordered.extrusion_loop.paths[..] else {
        panic!("a uniform-width loop keeps one sub-path");
    };
    // `make_clockwise` reverses the counter-clockwise source order.
    let points: Vec<(i64, i64)> = path
        .polyline
        .points
        .iter()
        .map(|point| (point.x, point.y))
        .collect();
    assert_eq!(
        points,
        vec![
            (0, 0),
            (0, 1_000_000),
            (1_000_000, 1_000_000),
            (1_000_000, 0),
            (0, 0),
        ]
    );
}

#[test]
fn internal_inset_uses_perimeter_role_and_flow() {
    let outcome = traverse_extrusions(vec![wall(1, true, true, &rectangle(420_000))], &context())
        .expect("internal walls convert with the perimeter flow");
    let [ExtrusionEntity::Loop(ordered)] = &outcome.collection.entities[..] else {
        panic!("a closed wall line appends one loop entity");
    };
    let [path] = &ordered.extrusion_loop.paths[..] else {
        panic!("a uniform-width loop keeps one sub-path");
    };
    assert_eq!(path.role, ExtrusionRole::Perimeter);
    assert_eq!(path.height, 0.3);
    let expected_width = 420_000_f32 * SCALE.factor() as f32 + 0.3 * ROUNDED_RECTANGLE_FACTOR;
    assert_eq!(path.width, expected_width);
}

#[test]
fn thin_wall_hole_reverses_entity_order_outside_outer_inner() {
    let walls = vec![
        wall(0, true, true, &rectangle(450_000)),
        wall(1, true, false, &rectangle(350_000)),
    ];
    let outcome = traverse_extrusions(walls, &context()).expect("two walls convert");
    let [ExtrusionEntity::Loop(hole), ExtrusionEntity::Loop(contour)] =
        &outcome.collection.entities[..]
    else {
        panic!("a thin wall keeps two loop entities");
    };
    // The hole prints first after `PerimeterGenerator.cpp:548` reversal.
    assert_eq!(hole.extrusion_loop.role, ExtrusionLoopRole::Hole);
    assert_eq!(contour.extrusion_loop.role, ExtrusionLoopRole::Default);
}

#[test]
fn thin_wall_hole_keeps_source_order_for_outer_inner_sequence() {
    let walls = vec![
        wall(0, true, true, &rectangle(450_000)),
        wall(1, true, false, &rectangle(350_000)),
    ];
    let mut context = context();
    context.wall_sequence = ProcessWallSequence::OuterInner;
    let outcome = traverse_extrusions(walls, &context).expect("two walls convert");
    let [ExtrusionEntity::Loop(contour), ExtrusionEntity::Loop(hole)] =
        &outcome.collection.entities[..]
    else {
        panic!("a thin wall keeps two loop entities");
    };
    assert_eq!(contour.extrusion_loop.role, ExtrusionLoopRole::Default);
    assert_eq!(hole.extrusion_loop.role, ExtrusionLoopRole::Hole);
}

#[test]
fn open_line_appends_one_multi_path_entity() {
    let outcome = traverse_extrusions(
        vec![wall(
            1,
            false,
            true,
            &[
                (0, 0, 420_000),
                (1_000_000, 0, 420_000),
                (2_000_000, 0, 420_000),
            ],
        )],
        &context(),
    )
    .expect("the ordinary branch converts an open wall line");
    let [ExtrusionEntity::MultiPath(multi_path)] = &outcome.collection.entities[..] else {
        panic!("an open wall line appends multi-path entities");
    };
    assert_eq!(outcome.collection.entities[0].inset_idx(), -1);
    let [path] = &multi_path.paths[..] else {
        panic!("a uniform-width open line keeps one sub-path");
    };
    assert_eq!(path.role, ExtrusionRole::Perimeter);
    assert_eq!(path.polyline.points.len(), 3);
    assert_eq!(path.polyline.points[2].x, 2_000_000);
}

#[test]
fn variable_width_open_line_keeps_chained_sub_paths_in_one_entity() {
    let outcome = traverse_extrusions(
        vec![wall(
            1,
            false,
            true,
            &[(0, 0, 200_000), (4_000_000, 0, 400_000)],
        )],
        &context(),
    )
    .expect("the ordinary branch converts a variable-width open wall line");
    let [ExtrusionEntity::MultiPath(multi_path)] = &outcome.collection.entities[..] else {
        panic!("an open wall line appends multi-path entities");
    };
    // The 0.2mm to 0.4mm ramp splits at the 0.05mm tolerance into four
    // constant-width sub-paths that still chain end-to-start
    // (`VariableWidth.cpp:27-56`, `PerimeterGenerator.cpp:549-566`).
    assert_eq!(multi_path.paths.len(), 4);
    for pair in multi_path.paths.windows(2) {
        assert_eq!(
            pair[0].polyline.points.last(),
            pair[1].polyline.points.first()
        );
    }
    let widths: Vec<f32> = multi_path.paths.iter().map(|path| path.width).collect();
    assert!(
        widths.windows(2).all(|pair| pair[0] < pair[1]),
        "sub-path widths grow along the ramp: {widths:?}"
    );
}

#[test]
fn empty_wall_lines_are_skipped() {
    let outcome = traverse_extrusions(vec![wall(0, true, true, &[])], &context())
        .expect("empty wall lines skip conversion");
    assert!(outcome.collection.entities.is_empty());
}

#[test]
fn steep_overhang_flags_follow_odd_reversing_layers_without_detection() {
    for (layer_id, expected) in [(3, true), (2, false)] {
        let mut context = context();
        context.overhang_reverse = true;
        context.layer_id = layer_id;
        let outcome = traverse_extrusions(vec![wall(0, true, true, &rectangle(450_000))], &context)
            .expect("the ordinary branch never clips");
        assert_eq!(outcome.steep_overhang_contour, expected);
        assert_eq!(outcome.steep_overhang_hole, expected);
    }
}

#[test]
fn overhang_clipping_branch_fails_closed() {
    let mut context = context();
    context.detect_overhang_wall = true;
    let error = traverse_extrusions(vec![wall(0, true, true, &rectangle(450_000))], &context)
        .expect_err("the Z-interpolating overhang branch is a later seam");
    assert!(
        matches!(error, SliceError::UnsupportedProjectFeature(ref key) if key == "detect_overhang_wall")
    );
}

#[test]
fn active_fuzzy_skin_fails_closed() {
    let mut context = context();
    context.fuzzy_skin = active_fuzzy_skin();
    let error = traverse_extrusions(vec![wall(0, true, true, &rectangle(450_000))], &context)
        .expect_err("the junction fuzzifier is a later seam");
    assert!(matches!(error, SliceError::UnsupportedProjectFeature(ref key) if key == "fuzzy_skin"));
}
