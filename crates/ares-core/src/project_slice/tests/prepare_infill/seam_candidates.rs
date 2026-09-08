use crate::{
    geometry::CoordinateScale,
    project_slice::{
        fill_entities,
        perimeters::classic::{
            chained_loops::{ExtrusionLoop, ExtrusionLoopRole},
            entity_collections::{
                ExtrusionEntity, ExtrusionEntityCollection, OrderedExtrusionLoop,
            },
            materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
        },
        seam_candidates,
        tests::prepare_infill::group_fills::focused::fixture::graph,
    },
};

#[test]
fn task22o97_normalizes_clockwise_closed_points_and_preserves_signed_angles() {
    let collection = collection(vec![path(
        &[
            (0, 0),
            (0, 1_000_000),
            (1_000_000, 1_000_000),
            (1_000_000, 0),
            (0, 0),
        ],
        ExtrusionRole::ExternalPerimeter,
        0.42,
    )]);

    let collections = [collection];
    let layer = generate(&collections, 0.2, CoordinateScale::Normal, 0.4, 0.42);

    assert_eq!(layer.perimeters.len(), 1);
    assert_eq!(layer.points.len(), 5);
    assert_eq!(layer.points[0].position, layer.points[4].position);
    assert!(
        layer
            .points
            .iter()
            .all(|point| point.local_ccw_angle <= 0.0)
    );
    assert!(
        layer
            .points
            .iter()
            .all(|point| point.local_ccw_angle.is_finite())
    );
    assert_eq!(
        layer
            .points
            .iter()
            .map(|point| point.local_ccw_angle.to_bits())
            .collect::<Vec<_>>(),
        vec![3_217_625_051; 5]
    );
    assert_eq!(layer.perimeters[0].flow_width, 0.42);
}

#[test]
fn task22o97_mixed_external_overhang_loop_keeps_collect_points_and_external_flow() {
    let collection = collection(vec![
        path(
            &[(0, 0), (1_000_000, 0), (1_000_000, 1_000_000)],
            ExtrusionRole::ExternalPerimeter,
            0.44,
        ),
        path(
            &[(1_000_000, 1_000_000), (0, 1_000_000), (0, 0)],
            ExtrusionRole::OverhangPerimeter,
            0.61,
        ),
    ]);

    let collections = [collection];
    let layer = generate(&collections, 0.2, CoordinateScale::Normal, 0.4, 0.44);

    assert_eq!(layer.perimeters.len(), 1);
    assert_eq!(layer.points.len(), 6);
    assert_eq!(layer.points[2].position, layer.points[3].position);
    assert_eq!(layer.perimeters[0].flow_width, 0.44);
    assert_eq!(layer.perimeters[0].start_index, 0);
    assert_eq!(layer.perimeters[0].end_index, 6);
}

#[test]
fn task22o97_fallback_polygon_uses_region_external_flow_width() {
    let collection = collection(vec![path(
        &[(0, 0), (1_000_000, 0), (0, 0)],
        ExtrusionRole::OverhangPerimeter,
        0.61,
    )]);

    let collections = [collection];
    let layer = generate(&collections, 0.2, CoordinateScale::Normal, 0.4, 0.42);

    assert_eq!(layer.perimeters[0].flow_width, 0.42);
}

#[test]
fn task22o97_each_polygon_uses_its_corresponding_region_flow_width() {
    let fallback = [collection(vec![path(
        &[(0, 0), (1_000_000, 0), (0, 0)],
        ExtrusionRole::OverhangPerimeter,
        0.71,
    )])];
    let external = [collection(vec![path(
        &[(2_000_000, 0), (3_000_000, 0), (2_000_000, 0)],
        ExtrusionRole::ExternalPerimeter,
        0.83,
    )])];

    let layer = seam_candidates::generate_regions(
        &[
            seam_candidates::RegionPerimeters {
                collection: None,
                external_flow_width: 0.31,
            },
            seam_candidates::RegionPerimeters {
                collection: Some(&fallback[0]),
                external_flow_width: 0.42,
            },
            seam_candidates::RegionPerimeters {
                collection: Some(&external[0]),
                external_flow_width: 0.53,
            },
        ],
        0.2,
        CoordinateScale::Normal,
        0.4,
    );

    assert_eq!(
        layer
            .perimeters
            .iter()
            .map(|perimeter| perimeter.flow_width)
            .collect::<Vec<_>>(),
        vec![0.42, 0.53]
    );
}

#[test]
fn task22o97_ksr_candidate_topology_inventory_is_nonempty() {
    let prepared = fill_entities::prepare(graph()).unwrap();
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let source = &traversal.objects[0];
    let prelude = &source
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let (compensated, _) = prelude.as_parts();
    let (post_regions, _) = compensated.as_parts();
    let (plan, _, _) = post_regions.as_parts();
    let nozzle_diameter = traversal
        .resolved
        .views
        .full
        .project
        .print
        .nozzle_diameter
        .0[0]
        .0 as f32;
    assert_eq!(nozzle_diameter, 0.4);

    let mut inventory = (0_usize, 0_usize);
    for (entities, layer) in prepared.objects[0].iter().zip(&plan.layers) {
        let candidates = generate(
            &entities.perimeters,
            layer.slice_z as f32,
            traversal.scale,
            nozzle_diameter,
            external_flow_width(&entities.perimeters),
        );
        assert!(candidates.perimeters.iter().all(|perimeter| {
            perimeter.start_index < perimeter.end_index
                && perimeter.end_index <= candidates.points.len()
        }));
        inventory.0 += candidates.perimeters.len();
        inventory.1 += candidates.points.len();
    }
    assert!(inventory.0 > 0);
    assert!(inventory.1 >= inventory.0);

    fill_entities::dispose(prepared);
}

fn external_flow_width(collections: &[ExtrusionEntityCollection]) -> f32 {
    collections
        .iter()
        .flat_map(|collection| &collection.entities)
        .flat_map(|entity| match entity {
            ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
            ExtrusionEntity::MultiPath(multi_path) => &multi_path.paths,
        })
        .find(|path| path.role == ExtrusionRole::ExternalPerimeter)
        .expect("KSR layers have an external perimeter")
        .width
}

fn generate(
    collections: &[ExtrusionEntityCollection],
    z: f32,
    scale: CoordinateScale,
    angle_arm_mm: f32,
    external_flow_width: f32,
) -> seam_candidates::LayerSeamCandidates {
    let regions = collections
        .iter()
        .map(|collection| seam_candidates::RegionPerimeters {
            collection: Some(collection),
            external_flow_width,
        })
        .collect::<Vec<_>>();
    seam_candidates::generate_regions(&regions, z, scale, angle_arm_mm)
}

fn collection(paths: Vec<ExtrusionPath>) -> ExtrusionEntityCollection {
    ExtrusionEntityCollection {
        entities: vec![ExtrusionEntity::Loop(OrderedExtrusionLoop {
            extrusion_loop: ExtrusionLoop {
                paths,
                role: ExtrusionLoopRole::Default,
            },
            inset_idx: 0,
        })],
        source_order: 0,
    }
}

fn path(points: &[(i64, i64)], role: ExtrusionRole, width: f32) -> ExtrusionPath {
    ExtrusionPath {
        polyline: Polyline3 {
            points: points.iter().map(|&(x, y)| Point3 { x, y, z: 0 }).collect(),
            fitting: Vec::new(),
        },
        role,
        can_reverse: true,
        mm3_per_mm: 1.0,
        width,
        height: 0.2,
    }
}
