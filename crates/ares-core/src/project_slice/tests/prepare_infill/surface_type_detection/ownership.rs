use crate::{
    geometry::ExPolygon,
    project_slice::{
        perimeters::{
            self,
            classic::{
                entity_collections::ExtrusionEntityCollection, gap_extrusion::GapFillEntity,
            },
        },
        prepare_infill::surface_type_detection,
    },
};

use super::super::super::support::KsrArchive;

#[test]
fn task22o17_moves_unchanged_o16_allocations_and_rebuilds_fill_surfaces() {
    let source =
        perimeters::prepare_post_layer_region_perimeters(&KsrArchive::new().bytes()).unwrap();
    let predecessor = std::ptr::from_ref(source.predecessor.as_classic());
    let perimeter = source
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.perimeters)
        .map(collection_allocations)
        .collect::<Vec<_>>();
    let thin = source
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.thin_fills)
        .map(gap_allocations)
        .collect::<Vec<_>>();
    let boundaries = source
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.fill_expolygons)
        .map(expolygon_allocations)
        .collect::<Vec<_>>();
    let no_overlap = source
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.fill_no_overlap_expolygons)
        .map(expolygon_allocations)
        .collect::<Vec<_>>();
    let old_fill = source
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.fill_surfaces)
        .flat_map(|surface| expolygon_allocations(surface.as_parts().1))
        .collect::<Vec<_>>();

    let output = surface_type_detection::prepare(source).unwrap();
    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(
        output
            .objects
            .iter()
            .flat_map(|object| object.records.iter().flatten())
            .flat_map(|record| &record.perimeters)
            .map(collection_allocations)
            .collect::<Vec<_>>(),
        perimeter
    );
    assert_eq!(
        output
            .objects
            .iter()
            .flat_map(|object| object.records.iter().flatten())
            .flat_map(|record| &record.thin_fills)
            .map(gap_allocations)
            .collect::<Vec<_>>(),
        thin
    );
    assert_eq!(
        output
            .objects
            .iter()
            .flat_map(|object| object.records.iter().flatten())
            .flat_map(|record| &record.fill_expolygons)
            .map(expolygon_allocations)
            .collect::<Vec<_>>(),
        boundaries
    );
    assert_eq!(
        output
            .objects
            .iter()
            .flat_map(|object| object.records.iter().flatten())
            .flat_map(|record| &record.fill_no_overlap_expolygons)
            .map(expolygon_allocations)
            .collect::<Vec<_>>(),
        no_overlap
    );
    let rebuilt = output
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.fill_surfaces)
        .flat_map(|surface| expolygon_allocations(surface.as_parts().1))
        .collect::<Vec<_>>();
    assert!(rebuilt.iter().all(|pointer| !old_fill.contains(pointer)));
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
