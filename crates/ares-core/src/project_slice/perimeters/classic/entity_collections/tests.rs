use super::{
    super::{
        chained_loops::{ChainedLoopNode, ExtrusionLoop, ExtrusionLoopRole},
        materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
        shortest_path::chain_extrusion_loops,
        traversal::{LowerFlowRoute, PendingExtrusionRole, PendingLoopRole, TraversalSeed},
    },
    orientation::{is_counter_clockwise, orient_loop, reverse_loop},
    traverse::traverse_loops,
    types::ExtrusionEntity,
};
use crate::{
    ProcessWallDirection,
    geometry::{Point, Polygon},
};

#[test]
fn task22o9_loop_entity_chain_uses_zero_and_clears_reversals() {
    let loops = vec![
        square_loop(100, 0, ExtrusionLoopRole::Default),
        square_loop(1, 0, ExtrusionLoopRole::Default),
        square_loop(10, 0, ExtrusionLoopRole::Default),
    ];
    assert_eq!(
        chain_extrusion_loops(&loops),
        vec![(1, false), (2, false), (0, false)]
    );
}

#[test]
fn task22o9_symmetric_loop_ties_follow_source_observed_chain() {
    let loops = [0, -14_999_999, -29_999_999]
        .into_iter()
        .flat_map(|y| {
            [3, 15_000_003, 30_000_003]
                .into_iter()
                .map(move |x| square_loop(x, y, ExtrusionLoopRole::Default))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        chain_extrusion_loops(&loops),
        [0, 3, 6, 7, 8, 5, 2, 1, 4]
            .into_iter()
            .map(|index| (index, false))
            .collect::<Vec<_>>()
    );
}

#[test]
fn task22o9_orientation_reverses_paths_in_place_with_exact_loop_polygon() {
    let mut loop_ = square_loop(0, 0, ExtrusionLoopRole::Default);
    let allocation = loop_.paths[0].polyline.points.as_ptr();
    assert!(is_counter_clockwise(&loop_));
    orient_loop(
        &mut loop_,
        ProcessWallDirection::CounterClockwise,
        false,
        false,
    );
    assert!(!is_counter_clockwise(&loop_));
    assert_eq!(loop_.paths[0].polyline.points.as_ptr(), allocation);
    reverse_loop(&mut loop_);
    assert!(is_counter_clockwise(&loop_));
}

#[test]
fn task22o9_multi_path_reverse_reverses_each_polyline_then_path_order() {
    let mut loop_ = ExtrusionLoop {
        paths: vec![
            path(&[(0, 0), (4, 0), (4, 4)], ExtrusionRole::Perimeter),
            path(&[(4, 4), (0, 4), (0, 0)], ExtrusionRole::OverhangPerimeter),
        ],
        role: ExtrusionLoopRole::Default,
    };
    assert!(is_counter_clockwise(&loop_));
    let allocations = loop_
        .paths
        .iter()
        .map(|path| path.polyline.points.as_ptr())
        .collect::<Vec<_>>();
    reverse_loop(&mut loop_);
    assert_eq!(loop_.paths[0].role, ExtrusionRole::OverhangPerimeter);
    assert_eq!(loop_.paths[1].role, ExtrusionRole::Perimeter);
    assert_eq!(
        loop_.paths[0]
            .polyline
            .points
            .iter()
            .map(|point| (point.x, point.y))
            .collect::<Vec<_>>(),
        vec![(0, 0), (0, 4), (4, 4)]
    );
    assert_eq!(loop_.paths[0].polyline.points.as_ptr(), allocations[1]);
    assert_eq!(loop_.paths[1].polyline.points.as_ptr(), allocations[0]);
}

#[test]
fn task22o9_both_wall_directions_and_lone_hole_exception_are_exhaustive() {
    for (direction, contour_ccw, hole_ccw, lone_hole_ccw) in [
        (ProcessWallDirection::CounterClockwise, true, false, true),
        (ProcessWallDirection::Clockwise, false, true, false),
    ] {
        let mut contour = square_loop(0, 0, ExtrusionLoopRole::Default);
        orient_loop(&mut contour, direction, true, false);
        assert_eq!(is_counter_clockwise(&contour), contour_ccw);
        let mut hole = square_loop(0, 0, ExtrusionLoopRole::Hole);
        orient_loop(&mut hole, direction, false, false);
        assert_eq!(is_counter_clockwise(&hole), hole_ccw);
        let mut lone_hole = square_loop(0, 0, ExtrusionLoopRole::Hole);
        orient_loop(&mut lone_hole, direction, false, true);
        assert_eq!(is_counter_clockwise(&lone_hole), lone_hole_ccw);
    }
}

#[test]
fn task22o9_large_coordinate_orientation_keeps_clipper_f64_cast_order() {
    let base = 1_i64 << 60;
    let loop_ = ExtrusionLoop {
        paths: vec![path(
            &[
                (base, base),
                (base, base + 128),
                (base + 128, base + 128),
                (base + 128, base),
                (base, base),
            ],
            ExtrusionRole::Perimeter,
        )],
        role: ExtrusionLoopRole::Default,
    };
    assert!(is_counter_clockwise(&loop_));
}

#[test]
fn task22o9_preserves_source_compact_entity_against_original_loop_indexing() {
    let nodes = vec![
        ChainedLoopNode {
            extrusion_loop: None,
            children: vec![ChainedLoopNode {
                extrusion_loop: Some(square_loop(20, 0, ExtrusionLoopRole::Hole)),
                children: Vec::new(),
            }],
        },
        ChainedLoopNode {
            extrusion_loop: Some(square_loop(1, 0, ExtrusionLoopRole::Internal)),
            children: Vec::new(),
        },
    ];
    let seeds = vec![
        seed(false, 7, vec![seed(false, 8, Vec::new())]),
        seed(true, 2, Vec::new()),
    ];
    let collection = traverse_loops(nodes, &seeds, ProcessWallDirection::CounterClockwise);
    assert_eq!(collection.entities.len(), 2);
    let [ExtrusionEntity::Loop(first), ExtrusionEntity::Loop(second)] = &collection.entities[..]
    else {
        panic!("classic traversal emits only loop entities");
    };
    assert_eq!(first.inset_idx, 7);
    assert_eq!(first.extrusion_loop.role, ExtrusionLoopRole::Internal);
    assert_eq!(second.inset_idx, 8);
    assert_eq!(second.extrusion_loop.role, ExtrusionLoopRole::Hole);
}

#[test]
fn task22o9_contours_emit_children_first_and_holes_emit_parent_first() {
    let nodes = vec![ChainedLoopNode {
        extrusion_loop: Some(square_loop(1, 0, ExtrusionLoopRole::Default)),
        children: vec![ChainedLoopNode {
            extrusion_loop: Some(square_loop(10, 0, ExtrusionLoopRole::Hole)),
            children: vec![ChainedLoopNode {
                extrusion_loop: Some(square_loop(20, 0, ExtrusionLoopRole::Internal)),
                children: Vec::new(),
            }],
        }],
    }];
    let seeds = vec![seed(
        true,
        0,
        vec![seed(false, 1, vec![seed(true, 2, Vec::new())])],
    )];
    let collection = traverse_loops(nodes, &seeds, ProcessWallDirection::CounterClockwise);
    assert_eq!(
        collection
            .entities
            .iter()
            .map(|entity| entity.inset_idx())
            .collect::<Vec<_>>(),
        vec![1, 2, 0]
    );
}

#[test]
fn task22o9_deep_traversal_and_cleanup_fit_a_constrained_stack() {
    std::thread::Builder::new()
        .stack_size(crate::project_slice::CONSTRAINED_TEST_STACK_SIZE)
        .spawn(|| {
            let (node, root_seed) = deep_pair(2_000);
            let collection = traverse_loops(
                vec![node],
                std::slice::from_ref(&root_seed),
                ProcessWallDirection::CounterClockwise,
            );
            assert_eq!(collection.entities.len(), 2_000);
            drain_seeds(vec![root_seed]);
            drop(collection);

            let (mut skipped, skipped_seed) = deep_pair(2_000);
            skipped.extrusion_loop = None;
            let collection = traverse_loops(
                vec![skipped],
                std::slice::from_ref(&skipped_seed),
                ProcessWallDirection::CounterClockwise,
            );
            assert!(collection.entities.is_empty());
            drain_seeds(vec![skipped_seed]);
        })
        .unwrap()
        .join()
        .unwrap();
}

fn deep_pair(depth: u16) -> (ChainedLoopNode, TraversalSeed) {
    let mut node = ChainedLoopNode {
        extrusion_loop: Some(square_loop(1, 0, ExtrusionLoopRole::Default)),
        children: Vec::new(),
    };
    let mut root_seed = seed(true, 0, Vec::new());
    for depth in 1..depth {
        node = ChainedLoopNode {
            extrusion_loop: Some(square_loop(
                i64::from(depth) + 1,
                0,
                ExtrusionLoopRole::Default,
            )),
            children: vec![node],
        };
        root_seed = seed(true, depth, vec![root_seed]);
    }
    (node, root_seed)
}

fn square_loop(x: i64, y: i64, role: ExtrusionLoopRole) -> ExtrusionLoop {
    ExtrusionLoop {
        paths: vec![path(
            &[(x, y), (x + 4, y), (x + 4, y + 4), (x, y + 4), (x, y)],
            ExtrusionRole::Perimeter,
        )],
        role,
    }
}

fn path(points: &[(i64, i64)], role: ExtrusionRole) -> ExtrusionPath {
    ExtrusionPath {
        polyline: Polyline3 {
            points: points.iter().map(|&(x, y)| Point3 { x, y, z: 0 }).collect(),
            fitting: Vec::new(),
        },
        role,
        can_reverse: true,
        mm3_per_mm: 1.0,
        width: 0.4,
        height: 0.2,
    }
}

fn seed(is_contour: bool, depth: u16, children: Vec<TraversalSeed>) -> TraversalSeed {
    TraversalSeed {
        polygon: Polygon::new(vec![Point::new(0, 0), Point::new(4, 0), Point::new(4, 4)]),
        depth,
        is_contour,
        is_smaller_width_perimeter: false,
        extrusion_role: PendingExtrusionRole::Perimeter,
        loop_role: if is_contour {
            PendingLoopRole::Default
        } else {
            PendingLoopRole::Hole
        },
        route: LowerFlowRoute::Internal,
        width: 0.4,
        mm3_per_mm: 1.0,
        children,
    }
}

fn drain_seeds(mut seeds: Vec<TraversalSeed>) {
    while let Some(mut seed) = seeds.pop() {
        seeds.append(&mut seed.children);
    }
}
