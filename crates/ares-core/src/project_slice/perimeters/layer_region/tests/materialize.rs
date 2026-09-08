use crate::{
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        perimeters::{
            classic::{
                chained_loops::{ExtrusionLoop, ExtrusionLoopRole},
                entity_collections::{
                    ExtrusionEntity, ExtrusionEntityCollection, OrderedExtrusionLoop,
                },
                gap_extrusion::{GapFillCollection, GapFillEntity, PreparedGapExtrusionSurface},
                infill_boundary::PreparedInfillBoundaryRecord,
                materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
                perimeter_append::{
                    AppendedPerimeterCollections, InactiveOuterBrimReordering,
                    InactiveOverhangReorientation, InactivePostCollectionBranches,
                    InactiveWallReordering,
                },
            },
            layer_region::materialize_record,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

#[test]
fn task22o16_materializes_five_fields_in_source_append_order_and_moves_nested_storage() {
    let source = PreparedInfillBoundaryRecord {
        surfaces: vec![
            surface(
                1,
                vec![collection(10, vec![path(100, ExtrusionRole::Perimeter)])],
                vec![GapFillEntity::Path(path(400, ExtrusionRole::GapFill))],
            ),
            surface(
                2,
                Vec::new(),
                vec![GapFillEntity::Loop(vec![
                    path(500, ExtrusionRole::GapFill),
                    path(600, ExtrusionRole::GapFill),
                ])],
            ),
            surface(
                3,
                vec![
                    collection(20, Vec::new()),
                    collection(30, vec![path(300, ExtrusionRole::ExternalPerimeter)]),
                ],
                Vec::new(),
            ),
        ],
        fill_surfaces: vec![
            RegionSurface::internal_with_metadata(rectangle(1_000), 0.2, 2, 1.5, 3),
            RegionSurface::internal_with_metadata(rectangle(2_000), 0.4, 4, 2.5, 5),
        ],
        fill_no_overlap: vec![rectangle(3_000), rectangle(4_000)],
        overlap: Vec::new(),
    };
    let perimeter_allocations = source
        .surfaces
        .iter()
        .flat_map(|surface| &surface.appended.collections)
        .map(|collection| {
            (
                collection.entities.as_ptr(),
                collection.entities.first().map(|entity| {
                    let paths = match entity {
                        ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
                        ExtrusionEntity::MultiPath(_) => {
                            panic!("classic append keeps loop entities")
                        }
                    };
                    (paths.as_ptr(), paths[0].polyline.points.as_ptr())
                }),
            )
        })
        .collect::<Vec<_>>();
    let gap_allocations = source
        .surfaces
        .iter()
        .flat_map(|surface| &surface.gap_fill.entities)
        .map(|entity| match entity {
            GapFillEntity::Path(path) => (None, vec![path.polyline.points.as_ptr()]),
            GapFillEntity::Loop(paths) => (
                Some(paths.as_ptr()),
                paths
                    .iter()
                    .map(|path| path.polyline.points.as_ptr())
                    .collect(),
            ),
        })
        .collect::<Vec<_>>();
    let fill_vector = source.fill_surfaces.as_ptr();
    let fill_geometry = source
        .fill_surfaces
        .iter()
        .map(|surface| surface.as_parts().1.contour().points().as_ptr())
        .collect::<Vec<_>>();
    let no_overlap_vector = source.fill_no_overlap.as_ptr();
    let no_overlap_geometry = source
        .fill_no_overlap
        .iter()
        .map(|expolygon| expolygon.contour().points().as_ptr())
        .collect::<Vec<_>>();

    let output = materialize_record(source);

    assert_eq!(
        output
            .perimeters
            .iter()
            .map(|collection| collection.entities.len())
            .collect::<Vec<_>>(),
        [1, 0, 1]
    );
    assert_eq!(
        output
            .perimeters
            .iter()
            .filter_map(|collection| collection.entities.first())
            .map(|entity| entity.inset_idx())
            .collect::<Vec<_>>(),
        [10, 30]
    );
    assert_eq!(
        output
            .thin_fills
            .iter()
            .map(|entity| match entity {
                GapFillEntity::Path(path) => vec![path.polyline.points[0].x],
                GapFillEntity::Loop(paths) =>
                    paths.iter().map(|path| path.polyline.points[0].x).collect(),
            })
            .collect::<Vec<_>>(),
        [vec![400], vec![500, 600]]
    );
    let first_loop = collection_loop(&output.perimeters[0], 0);
    assert_eq!(first_loop.role, ExtrusionLoopRole::Internal);
    assert_eq!(
        (
            first_loop.paths[0].role,
            first_loop.paths[0].mm3_per_mm,
            first_loop.paths[0].width,
            first_loop.paths[0].height,
        ),
        (ExtrusionRole::Perimeter, 1.0, 0.1, 0.2)
    );
    let final_path = &collection_loop(&output.perimeters[2], 0).paths[0];
    assert_eq!(
        (final_path.role, final_path.mm3_per_mm, final_path.width),
        (ExtrusionRole::ExternalPerimeter, 3.0, 0.3)
    );
    for entity in &output.thin_fills {
        let paths = match entity {
            GapFillEntity::Path(path) => std::slice::from_ref(path),
            GapFillEntity::Loop(paths) => paths,
        };
        assert!(paths.iter().all(|path| {
            path.role == ExtrusionRole::GapFill
                && path.mm3_per_mm == path.polyline.points[0].x as f64 / 100.0
                && path.width == path.polyline.points[0].x as f32 / 1_000.0
                && path.height == 0.2
        }));
    }
    for (collection, expected) in output.perimeters.iter().zip(perimeter_allocations) {
        assert_eq!(collection.entities.as_ptr(), expected.0);
        if let (Some(entity), Some((paths, points))) = (collection.entities.first(), expected.1) {
            let entity_paths = match entity {
                ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
                ExtrusionEntity::MultiPath(_) => panic!("classic append keeps loop entities"),
            };
            assert_eq!(entity_paths.as_ptr(), paths);
            assert_eq!(entity_paths[0].polyline.points.as_ptr(), points);
        }
    }
    for (entity, (paths, points)) in output.thin_fills.iter().zip(gap_allocations) {
        let actual = match entity {
            GapFillEntity::Path(path) => (None, vec![path.polyline.points.as_ptr()]),
            GapFillEntity::Loop(paths) => (
                Some(paths.as_ptr()),
                paths
                    .iter()
                    .map(|path| path.polyline.points.as_ptr())
                    .collect(),
            ),
        };
        assert_eq!(actual, (paths, points));
    }
    assert_eq!(output.fill_surfaces.as_ptr(), fill_vector);
    assert_eq!(
        output.fill_no_overlap_expolygons.as_ptr(),
        no_overlap_vector
    );
    for (surface, expected) in output.fill_surfaces.iter().zip(fill_geometry) {
        assert_eq!(surface.as_parts().1.contour().points().as_ptr(), expected);
    }
    for (expolygon, expected) in output
        .fill_no_overlap_expolygons
        .iter()
        .zip(no_overlap_geometry)
    {
        assert_eq!(expolygon.contour().points().as_ptr(), expected);
    }
    assert_eq!(output.fill_expolygons.len(), output.fill_surfaces.len());
    for ((copied, surface), expected) in output
        .fill_expolygons
        .iter()
        .zip(&output.fill_surfaces)
        .zip([(0.2, 2, 1.5, 3), (0.4, 4, 2.5, 5)])
    {
        let (kind, source, thickness, layers, angle, extra) = surface.as_parts();
        assert_eq!(kind, RegionSurfaceKind::Internal);
        assert_eq!((thickness, layers, angle, extra), expected);
        assert_eq!(copied, source);
        assert_ne!(
            copied.contour().points().as_ptr(),
            source.contour().points().as_ptr()
        );
    }
}

#[test]
fn task22o16_empty_surface_record_keeps_record_fill_order() {
    let source = PreparedInfillBoundaryRecord {
        surfaces: Vec::new(),
        fill_surfaces: vec![
            RegionSurface::internal(rectangle(20)),
            RegionSurface::internal(rectangle(10)),
        ],
        fill_no_overlap: vec![rectangle(40), rectangle(30)],
        overlap: Vec::new(),
    };
    let output = materialize_record(source);
    assert!(output.perimeters.is_empty());
    assert!(output.thin_fills.is_empty());
    assert_eq!(first_x_surfaces(&output.fill_surfaces), [20, 10]);
    assert_eq!(first_x(&output.fill_expolygons), [20, 10]);
    assert_eq!(first_x(&output.fill_no_overlap_expolygons), [40, 30]);
}

fn surface(
    source_index: usize,
    collections: Vec<ExtrusionEntityCollection>,
    entities: Vec<GapFillEntity>,
) -> PreparedGapExtrusionSurface {
    PreparedGapExtrusionSurface {
        source_index,
        inactive: InactivePostCollectionBranches {
            overhang_reorientation: InactiveOverhangReorientation::Disabled {
                overhang_reverse_internal_only: false,
            },
            wall_reordering: InactiveWallReordering::InnerOuter {
                outer_brim: InactiveOuterBrimReordering::WidthNotPositive { brim_width: 0.0 },
            },
        },
        appended: AppendedPerimeterCollections { collections },
        medial: None,
        gap_fill: GapFillCollection { entities },
        remaining: Vec::new(),
    }
}

fn collection_loop(collection: &ExtrusionEntityCollection, index: usize) -> &ExtrusionLoop {
    match &collection.entities[index] {
        ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop,
        ExtrusionEntity::MultiPath(_) => panic!("classic append keeps loop entities"),
    }
}

fn collection(inset_idx: i32, paths: Vec<ExtrusionPath>) -> ExtrusionEntityCollection {
    ExtrusionEntityCollection {
        entities: paths
            .into_iter()
            .map(|path| {
                ExtrusionEntity::Loop(OrderedExtrusionLoop {
                    extrusion_loop: ExtrusionLoop {
                        paths: vec![path],
                        role: ExtrusionLoopRole::Internal,
                    },
                    inset_idx,
                })
            })
            .collect(),
        source_order: 0,
    }
}

fn path(x: i64, role: ExtrusionRole) -> ExtrusionPath {
    ExtrusionPath {
        polyline: Polyline3 {
            points: vec![
                Point3 { x, y: x + 1, z: 0 },
                Point3 {
                    x: x + 2,
                    y: x + 3,
                    z: 0,
                },
            ],
            fitting: Vec::new(),
        },
        role,
        can_reverse: true,
        mm3_per_mm: x as f64 / 100.0,
        width: x as f32 / 1_000.0,
        height: 0.2,
    }
}

fn rectangle(x: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(x, 0),
            Point::new(x + 5, 0),
            Point::new(x + 5, 5),
            Point::new(x, 5),
        ]),
        Vec::new(),
    )
}

fn first_x_surfaces(surfaces: &[RegionSurface]) -> Vec<i64> {
    surfaces
        .iter()
        .map(|surface| surface.as_parts().1.contour().points()[0].x())
        .collect()
}

fn first_x(expolygons: &[ExPolygon]) -> Vec<i64> {
    expolygons
        .iter()
        .map(|expolygon| expolygon.contour().points()[0].x())
        .collect()
}
