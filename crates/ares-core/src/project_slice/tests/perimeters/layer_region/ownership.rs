use crate::{
    geometry::ExPolygon,
    project_slice::perimeters::{
        classic::{
            entity_collections::ExtrusionEntityCollection, gap_extrusion::GapFillEntity,
            infill_boundary::PreparedInfillBoundaryObject,
        },
        layer_region, prepare_post_classic_infill_boundary,
    },
};

use super::super::super::support::ksr_project;

#[test]
fn task22o16_moves_only_source_layer_region_allocations_and_keeps_boxed_context() {
    let source = prepare_post_classic_infill_boundary(ksr_project()).unwrap();
    let predecessor = std::ptr::from_ref(source.predecessor.as_ref());
    let perimeters = source_perimeter_allocations(&source.objects);
    let gaps = source_gap_allocations(&source.objects);
    let fill_vectors = source
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .map(|record| record.fill_surfaces.as_ptr() as usize)
        .collect::<Vec<_>>();
    let fill_geometry = source
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.fill_surfaces)
        .map(|surface| expolygon_allocations(surface.as_parts().1))
        .collect::<Vec<_>>();
    let no_overlap_vectors = source
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .map(|record| record.fill_no_overlap.as_ptr() as usize)
        .collect::<Vec<_>>();
    let no_overlap_geometry = source
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.fill_no_overlap)
        .map(expolygon_allocations)
        .collect::<Vec<_>>();

    let output = layer_region::finish(source);

    assert_eq!(
        std::ptr::from_ref(output.predecessor.as_classic()),
        predecessor
    );
    assert_eq!(output_perimeter_allocations(&output.objects), perimeters);
    assert_eq!(output_gap_allocations(&output.objects), gaps);
    assert_eq!(
        output
            .objects
            .iter()
            .flat_map(|object| object.records.iter().flatten())
            .map(|record| record.fill_surfaces.as_ptr() as usize)
            .collect::<Vec<_>>(),
        fill_vectors
    );
    assert_eq!(
        output
            .objects
            .iter()
            .flat_map(|object| object.records.iter().flatten())
            .flat_map(|record| &record.fill_surfaces)
            .map(|surface| expolygon_allocations(surface.as_parts().1))
            .collect::<Vec<_>>(),
        fill_geometry
    );
    assert_eq!(
        output
            .objects
            .iter()
            .flat_map(|object| object.records.iter().flatten())
            .map(|record| record.fill_no_overlap_expolygons.as_ptr() as usize)
            .collect::<Vec<_>>(),
        no_overlap_vectors
    );
    assert_eq!(
        output
            .objects
            .iter()
            .flat_map(|object| object.records.iter().flatten())
            .flat_map(|record| &record.fill_no_overlap_expolygons)
            .map(expolygon_allocations)
            .collect::<Vec<_>>(),
        no_overlap_geometry
    );
    for record in output
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
    {
        for (copied, surface) in record.fill_expolygons.iter().zip(&record.fill_surfaces) {
            let source = surface.as_parts().1;
            assert_eq!(copied, source);
            assert_ne!(
                copied.contour().points().as_ptr(),
                source.contour().points().as_ptr()
            );
            for (copied_hole, source_hole) in copied.holes().iter().zip(source.holes()) {
                assert_ne!(copied_hole.points().as_ptr(), source_hole.points().as_ptr());
            }
        }
    }
}

fn source_perimeter_allocations(objects: &[PreparedInfillBoundaryObject]) -> Vec<Vec<usize>> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .flat_map(|surface| &surface.appended.collections)
        .map(collection_allocations)
        .collect()
}

fn output_perimeter_allocations(
    objects: &[layer_region::PreparedLayerRegionPerimeterObject],
) -> Vec<Vec<usize>> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.perimeters)
        .map(collection_allocations)
        .collect()
}

fn collection_allocations(collection: &ExtrusionEntityCollection) -> Vec<usize> {
    std::iter::once(collection.entities.as_ptr() as usize)
        .chain(collection.entities.iter().flat_map(|entity| {
            std::iter::once(entity.extrusion_loop.paths.as_ptr() as usize).chain(
                entity
                    .extrusion_loop
                    .paths
                    .iter()
                    .map(|path| path.polyline.points.as_ptr() as usize),
            )
        }))
        .collect()
}

fn source_gap_allocations(objects: &[PreparedInfillBoundaryObject]) -> Vec<Vec<usize>> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .flat_map(|surface| &surface.gap_fill.entities)
        .map(gap_allocations)
        .collect()
}

fn output_gap_allocations(
    objects: &[layer_region::PreparedLayerRegionPerimeterObject],
) -> Vec<Vec<usize>> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.thin_fills)
        .map(gap_allocations)
        .collect()
}

fn gap_allocations(entity: &GapFillEntity) -> Vec<usize> {
    match entity {
        GapFillEntity::Path(path) => vec![path.polyline.points.as_ptr() as usize],
        GapFillEntity::Loop(paths) => std::iter::once(paths.as_ptr() as usize)
            .chain(
                paths
                    .iter()
                    .map(|path| path.polyline.points.as_ptr() as usize),
            )
            .collect(),
    }
}

fn expolygon_allocations(expolygon: &ExPolygon) -> Vec<usize> {
    std::iter::once(expolygon.contour().points().as_ptr() as usize)
        .chain(
            expolygon
                .holes()
                .iter()
                .map(|hole| hole.points().as_ptr() as usize),
        )
        .collect()
}
