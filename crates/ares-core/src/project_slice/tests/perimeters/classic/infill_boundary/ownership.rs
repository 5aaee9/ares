use crate::project_slice::perimeters::{
    classic::{
        entity_collections::ExtrusionEntity,
        gap_extrusion::{GapFillEntity, PreparedGapExtrusionSurface},
        infill_boundary,
    },
    prepare_post_classic_gap_extrusion,
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o15_moves_every_o14_surface_and_boxed_predecessor_allocation() {
    let source = prepare_post_classic_gap_extrusion(ksr_project()).unwrap();
    let predecessor = std::ptr::from_ref(source.predecessor.as_ref());
    let surface_vectors = source
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .map(|record| record.surfaces.as_ptr() as usize)
        .collect::<Vec<_>>();
    let expected_allocations = allocations(source_surfaces(&source.objects));
    let expected_content = content(source_surfaces(&source.objects));

    let output = infill_boundary::finish(source).unwrap();

    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(
        output
            .objects
            .iter()
            .flat_map(|object| object.records.iter().flatten())
            .map(|record| record.surfaces.as_ptr() as usize)
            .collect::<Vec<_>>(),
        surface_vectors,
    );
    assert_eq!(
        allocations(output_surfaces(&output.objects)),
        expected_allocations,
    );
    assert_eq!(content(output_surfaces(&output.objects)), expected_content);
}

fn source_surfaces(
    objects: &[crate::project_slice::perimeters::classic::gap_extrusion::PreparedGapExtrusionObject],
) -> impl Iterator<Item = &PreparedGapExtrusionSurface> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
}

fn output_surfaces(
    objects: &[crate::project_slice::perimeters::classic::infill_boundary::PreparedInfillBoundaryObject],
) -> impl Iterator<Item = &PreparedGapExtrusionSurface> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
}

fn allocations<'a>(surfaces: impl Iterator<Item = &'a PreparedGapExtrusionSurface>) -> Vec<usize> {
    surfaces
        .flat_map(|surface| {
            let remaining = std::iter::once(surface.remaining.as_ptr() as usize)
                .chain(surface.remaining.iter().flat_map(expolygon_allocations));
            let gap = std::iter::once(surface.gap_fill.entities.as_ptr() as usize).chain(
                surface
                    .gap_fill
                    .entities
                    .iter()
                    .flat_map(entity_allocations),
            );
            let o10 = collection_allocations(&surface.appended.collections);
            let medial = surface.medial.iter().flat_map(|domain| {
                std::iter::once(domain.predecessor.expolygons.as_ptr() as usize)
                    .chain(
                        domain
                            .predecessor
                            .expolygons
                            .iter()
                            .flat_map(expolygon_allocations),
                    )
                    .chain(domain.polylines.iter().flat_map(|polyline| {
                        [
                            polyline.points.as_ptr() as usize,
                            polyline.width.as_ptr() as usize,
                        ]
                    }))
            });
            remaining.chain(gap).chain(medial).chain(o10)
        })
        .collect()
}

fn collection_allocations(
    collections: &[crate::project_slice::perimeters::classic::entity_collections::ExtrusionEntityCollection],
) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(collections.as_ptr() as usize).chain(collections.iter().flat_map(
        |collection| {
            std::iter::once(collection.entities.as_ptr() as usize).chain(
                collection.entities.iter().flat_map(|entity| {
                    let paths = match entity {
                        ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
                        ExtrusionEntity::MultiPath(_) => {
                            panic!("classic collections keep loop entities")
                        }
                    };
                    std::iter::once(paths.as_ptr() as usize).chain(
                        paths
                            .iter()
                            .map(|path| path.polyline.points.as_ptr() as usize),
                    )
                }),
            )
        },
    ))
}

fn expolygon_allocations(
    expolygon: &crate::geometry::ExPolygon,
) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(expolygon.contour().points().as_ptr() as usize).chain(
        expolygon
            .holes()
            .iter()
            .map(|hole| hole.points().as_ptr() as usize),
    )
}

fn entity_allocations(entity: &GapFillEntity) -> impl Iterator<Item = usize> + '_ {
    let paths = match entity {
        GapFillEntity::Path(path) => std::slice::from_ref(path),
        GapFillEntity::Loop(paths) => paths,
    };
    std::iter::once(paths.as_ptr() as usize).chain(
        paths
            .iter()
            .map(|path| path.polyline.points.as_ptr() as usize),
    )
}

fn content<'a>(surfaces: impl Iterator<Item = &'a PreparedGapExtrusionSurface>) -> Vec<String> {
    surfaces.map(|surface| format!("{surface:?}")).collect()
}
