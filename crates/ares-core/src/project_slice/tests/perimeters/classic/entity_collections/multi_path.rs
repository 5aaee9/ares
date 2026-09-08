// Arachne integration seam 2: the typed extrusion entity vocabulary from
// OrcaSlicer `ExtrusionEntity.hpp` — `ExtrusionMultiPath` open variable-width
// entities (`PerimeterGenerator.cpp:553-566`) coexisting with classic ordered
// loops (`PerimeterGenerator.cpp:270`) inside one
// `ExtrusionEntityCollection`.

use crate::{
    geometry::CoordinateScale,
    project_slice::{
        perimeters::classic::{
            chained_loops::{ExtrusionLoop, ExtrusionLoopRole},
            entity_collections::{
                ExtrusionEntity, ExtrusionEntityCollection, ExtrusionMultiPath,
                OrderedExtrusionLoop,
            },
            materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
        },
        seam_candidates::{self, RegionPerimeters},
    },
};

fn path(points: &[(i64, i64)], role: ExtrusionRole, width: f32) -> ExtrusionPath {
    ExtrusionPath {
        polyline: Polyline3 {
            points: points.iter().map(|&(x, y)| Point3 { x, y, z: 0 }).collect(),
            fitting: Vec::new(),
        },
        role,
        can_reverse: true,
        mm3_per_mm: f64::from(width) / 2.0,
        width,
        height: 0.2,
    }
}

fn ordered_loop(role: ExtrusionRole, inset_idx: i32) -> ExtrusionEntity {
    ExtrusionEntity::Loop(OrderedExtrusionLoop {
        extrusion_loop: ExtrusionLoop {
            paths: vec![path(
                &[(0, 0), (1_000, 0), (1_000, 1_000), (0, 1_000), (0, 0)],
                role,
                0.45,
            )],
            role: if role == ExtrusionRole::ExternalPerimeter {
                ExtrusionLoopRole::Default
            } else {
                ExtrusionLoopRole::Internal
            },
        },
        inset_idx,
    })
}

#[test]
fn collection_holds_loops_and_multi_paths_in_source_order() {
    let multi_path = || {
        ExtrusionEntity::MultiPath(ExtrusionMultiPath {
            paths: vec![
                path(&[(0, 0), (500, 0)], ExtrusionRole::ExternalPerimeter, 0.45),
                path(&[(500, 0), (500, 500)], ExtrusionRole::Perimeter, 0.42),
            ],
        })
    };
    let collection = ExtrusionEntityCollection {
        entities: vec![ordered_loop(ExtrusionRole::Perimeter, 1), multi_path()],
        source_order: 3,
    };

    assert_eq!(collection.entities.len(), 2);
    assert_eq!(
        collection.entities[0],
        ordered_loop(ExtrusionRole::Perimeter, 1)
    );
    assert_eq!(collection.entities[1], multi_path());
    assert_eq!(collection.source_order, 3);
}

#[test]
fn multi_path_retains_variable_width_sub_path_flows() {
    // `traverse_extrusions` clips a variable-width wall line into
    // constant-width sub-paths, each with its own flow and width
    // (`PerimeterGenerator.cpp:553-566`, `ExtrusionEntity.hpp`).
    let ExtrusionEntity::MultiPath(multi_path) = ExtrusionEntity::MultiPath(ExtrusionMultiPath {
        paths: vec![
            path(
                &[(0, 0), (1_000, 0)],
                ExtrusionRole::ExternalPerimeter,
                0.45,
            ),
            path(&[(1_000, 0), (1_000, 900)], ExtrusionRole::Perimeter, 0.30),
        ],
    }) else {
        panic!("multi-path vocabulary stays constructible");
    };

    assert_eq!(multi_path.paths.len(), 2);
    assert_eq!(multi_path.paths[0].width, 0.45);
    assert_eq!(multi_path.paths[0].mm3_per_mm, f64::from(0.45_f32) / 2.0);
    assert_eq!(multi_path.paths[0].role, ExtrusionRole::ExternalPerimeter);
    assert_eq!(multi_path.paths[1].width, 0.30);
    assert_eq!(multi_path.paths[1].mm3_per_mm, f64::from(0.30_f32) / 2.0);
    assert_eq!(multi_path.paths[1].role, ExtrusionRole::Perimeter);
}

#[test]
fn inset_idx_keeps_loop_depth_and_base_default_for_multi_paths() {
    // `ExtrusionEntity::inset_idx` defaults to -1 (`ExtrusionEntity.hpp`);
    // classic loops set it from loop depth (`PerimeterGenerator.cpp:270`).
    assert_eq!(ordered_loop(ExtrusionRole::Perimeter, 4).inset_idx(), 4);
    assert_eq!(
        ExtrusionEntity::MultiPath(ExtrusionMultiPath { paths: vec![] }).inset_idx(),
        -1
    );
}

#[test]
fn first_point3_reads_the_first_sub_path_point_for_both_kinds() {
    // `ExtrusionEntity::first_point` (`ExtrusionEntity.hpp`) is the first
    // point of the first sub-path for loops and multi-paths alike.
    let multi_path = ExtrusionEntity::MultiPath(ExtrusionMultiPath {
        paths: vec![
            path(&[(70, 0), (500, 0)], ExtrusionRole::ExternalPerimeter, 0.45),
            path(&[(500, 0), (500, 500)], ExtrusionRole::Perimeter, 0.42),
        ],
    });

    let expected_loop = Point3 { x: 0, y: 0, z: 0 };
    let expected_multi_path = Point3 { x: 70, y: 0, z: 0 };
    assert_eq!(
        *ordered_loop(ExtrusionRole::Perimeter, 0).first_point3(),
        expected_loop
    );
    assert_eq!(*multi_path.first_point3(), expected_multi_path);
}

#[test]
fn seam_extraction_reads_multi_path_points_only_through_entity_role() {
    // `SeamPlacer.cpp:413-420`: only loops scan sub-paths for the external
    // role; a multi-path contributes through its entity role, the first
    // sub-path's role (`ExtrusionEntity.hpp`).
    let external_multi_path = ExtrusionEntity::MultiPath(ExtrusionMultiPath {
        paths: vec![
            path(
                &[(0, 0), (1_000, 0), (1_000, 700)],
                ExtrusionRole::ExternalPerimeter,
                0.45,
            ),
            path(
                &[(1_000, 700), (1_000, 900)],
                ExtrusionRole::Perimeter,
                0.42,
            ),
        ],
    });
    let collection = ExtrusionEntityCollection {
        entities: vec![external_multi_path],
        source_order: 0,
    };
    let candidates = seam_candidates::generate_regions(
        &[RegionPerimeters {
            collection: Some(&collection),
            external_flow_width: 0.45,
        }],
        0.2,
        CoordinateScale::Normal,
        0.0,
    );

    assert_eq!(candidates.perimeters.len(), 1);
    // Both sub-paths contribute points (`ExtrusionEntity::collect_points`).
    assert_eq!(candidates.points.len(), 5);
    assert_eq!(candidates.perimeters[0].flow_width, 0.45);

    // A collection whose loop is already external never falls back, so a
    // perimeter-role multi-path contributes nothing beside it.
    let mixed = ExtrusionEntityCollection {
        entities: vec![
            ordered_loop(ExtrusionRole::ExternalPerimeter, 0),
            ExtrusionEntity::MultiPath(ExtrusionMultiPath {
                paths: vec![path(
                    &[(9_000, 9_000), (9_100, 9_100)],
                    ExtrusionRole::Perimeter,
                    0.42,
                )],
            }),
        ],
        source_order: 0,
    };
    let candidates = seam_candidates::generate_regions(
        &[RegionPerimeters {
            collection: Some(&mixed),
            external_flow_width: 0.45,
        }],
        0.2,
        CoordinateScale::Normal,
        0.0,
    );

    assert_eq!(candidates.perimeters.len(), 1);
    assert_eq!(candidates.points.len(), 5);
}
