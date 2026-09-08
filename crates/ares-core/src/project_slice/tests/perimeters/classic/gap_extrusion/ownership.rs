use crate::project_slice::perimeters::{
    classic::{
        entity_collections::ExtrusionEntity,
        gap_extrusion::{self, PreparedGapExtrusionObject},
        medial_gap::PreparedMedialGapObject,
    },
    prepare_post_classic_medial_gap,
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o14_preserves_o13_o11_o10_and_boxed_o5_allocations() {
    let source = prepare_post_classic_medial_gap(ksr_project()).unwrap();
    let predecessor = std::ptr::from_ref(source.predecessor.as_ref());
    let allocations = source_allocations(&source.objects);
    let nested_content = source_content(&source.objects);

    let output = gap_extrusion::finish(source).unwrap();

    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(output_allocations(&output.objects), allocations);
    assert_eq!(output_content(&output.objects), nested_content);
}

fn source_allocations(objects: &[PreparedMedialGapObject]) -> Vec<usize> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .flat_map(|surface| {
            medial_allocations(surface.medial.as_ref())
                .chain(collection_allocations(&surface.appended.collections))
        })
        .collect()
}

fn output_allocations(objects: &[PreparedGapExtrusionObject]) -> Vec<usize> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .flat_map(|surface| {
            medial_allocations(surface.medial.as_ref())
                .chain(collection_allocations(&surface.appended.collections))
        })
        .collect()
}

fn medial_allocations(
    medial: Option<&crate::project_slice::perimeters::classic::medial_gap::MedialGapDomain>,
) -> impl Iterator<Item = usize> + '_ {
    medial.into_iter().flat_map(|domain| {
        std::iter::once(domain.predecessor.expolygons.as_ptr() as usize)
            .chain(domain.predecessor.expolygons.iter().flat_map(|expolygon| {
                std::iter::once(expolygon.contour().points().as_ptr() as usize).chain(
                    expolygon
                        .holes()
                        .iter()
                        .map(|hole| hole.points().as_ptr() as usize),
                )
            }))
            .chain(domain.polylines.iter().flat_map(|polyline| {
                [
                    polyline.points.as_ptr() as usize,
                    polyline.width.as_ptr() as usize,
                ]
            }))
    })
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

fn source_content(objects: &[PreparedMedialGapObject]) -> Vec<String> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .map(|surface| {
            format!(
                "{:?}|{:?}|{:?}|{:?}",
                surface.source_index, surface.inactive, surface.appended, surface.medial
            )
        })
        .collect()
}

fn output_content(objects: &[PreparedGapExtrusionObject]) -> Vec<String> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .map(|surface| {
            format!(
                "{:?}|{:?}|{:?}|{:?}",
                surface.source_index, surface.inactive, surface.appended, surface.medial
            )
        })
        .collect()
}
