use crate::project_slice::perimeters::{
    classic::{entity_collections::ExtrusionEntity, medial_gap},
    prepare_post_classic_gap_domain, prepare_post_classic_medial_gap,
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o13_direct_preserves_none_some_and_well_formed_aggregation() {
    let output = prepare_post_classic_medial_gap(ksr_project()).unwrap();
    let mut absent = 0;
    let mut present = 0;
    let surfaces = output
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces);
    for surface in surfaces {
        let Some(domain) = &surface.medial else {
            absent += 1;
            continue;
        };
        present += 1;
        for polyline in &domain.polylines {
            assert!(polyline.points.len() >= 2);
            assert_eq!(polyline.width.len(), (polyline.points.len() - 1) * 2);
        }
    }
    assert!(absent > 0);
    assert!(present > 0);
}

#[test]
fn task22o13_literal_some_empty_remains_present_and_empty() {
    let mut source = prepare_post_classic_gap_domain(ksr_project()).unwrap();
    let surface = source
        .objects
        .iter_mut()
        .flat_map(|object| object.records.iter_mut().flatten())
        .flat_map(|record| &mut record.surfaces)
        .find(|surface| surface.pre_medial.is_some())
        .unwrap();
    surface.pre_medial.as_mut().unwrap().expolygons.clear();
    let output = medial_gap::finish(source).unwrap();
    let domain = output
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .filter_map(|surface| surface.medial.as_ref())
        .find(|domain| domain.predecessor.expolygons.is_empty())
        .unwrap();
    assert!(domain.predecessor.expolygons.is_empty());
    assert!(domain.polylines.is_empty());
}

#[test]
fn task22o13_finish_preserves_o11_polygon_and_o10_collection_allocations() {
    let source = prepare_post_classic_gap_domain(ksr_project()).unwrap();
    let predecessor = std::ptr::from_ref(source.predecessor.as_ref());
    let polygon_allocations = polygon_allocations_gap(&source.objects);
    let collection_allocations = collection_allocations_gap(&source.objects);
    let nested_content = nested_content_gap(&source.objects);
    let output = medial_gap::finish(source).unwrap();
    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(
        polygon_allocations_medial(&output.objects),
        polygon_allocations
    );
    assert_eq!(
        collection_allocations_medial(&output.objects),
        collection_allocations
    );
    assert_eq!(nested_content_medial(&output.objects), nested_content);
}

fn nested_content_gap(
    objects: &[crate::project_slice::perimeters::classic::gap_domain::PreparedGapDomainObject],
) -> Vec<String> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .map(|surface| {
            format!(
                "{:?}|{:?}|{:?}",
                surface.inactive, surface.appended, surface.pre_medial
            )
        })
        .collect()
}

fn nested_content_medial(
    objects: &[crate::project_slice::perimeters::classic::medial_gap::PreparedMedialGapObject],
) -> Vec<String> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .map(|surface| {
            format!(
                "{:?}|{:?}|{:?}",
                surface.inactive,
                surface.appended,
                surface.medial.as_ref().map(|domain| &domain.predecessor)
            )
        })
        .collect()
}

fn polygon_allocations_gap(
    objects: &[crate::project_slice::perimeters::classic::gap_domain::PreparedGapDomainObject],
) -> Vec<usize> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .filter_map(|surface| surface.pre_medial.as_ref())
        .flat_map(|domain| {
            std::iter::once(domain.expolygons.as_ptr() as usize).chain(
                domain.expolygons.iter().flat_map(|expolygon| {
                    std::iter::once(expolygon.contour().points().as_ptr() as usize).chain(
                        expolygon
                            .holes()
                            .iter()
                            .map(|hole| hole.points().as_ptr() as usize),
                    )
                }),
            )
        })
        .collect()
}

fn polygon_allocations_medial(
    objects: &[crate::project_slice::perimeters::classic::medial_gap::PreparedMedialGapObject],
) -> Vec<usize> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .filter_map(|surface| surface.medial.as_ref())
        .flat_map(|domain| {
            std::iter::once(domain.predecessor.expolygons.as_ptr() as usize).chain(
                domain.predecessor.expolygons.iter().flat_map(|expolygon| {
                    std::iter::once(expolygon.contour().points().as_ptr() as usize).chain(
                        expolygon
                            .holes()
                            .iter()
                            .map(|hole| hole.points().as_ptr() as usize),
                    )
                }),
            )
        })
        .collect()
}

fn collection_allocations_gap(
    objects: &[crate::project_slice::perimeters::classic::gap_domain::PreparedGapDomainObject],
) -> Vec<usize> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .flat_map(|surface| {
            std::iter::once(surface.appended.collections.as_ptr() as usize).chain(
                surface.appended.collections.iter().flat_map(|collection| {
                    std::iter::once(collection.entities.as_ptr() as usize)
                        .chain(collection.entities.iter().flat_map(loop_path_allocations))
                }),
            )
        })
        .collect()
}

fn collection_allocations_medial(
    objects: &[crate::project_slice::perimeters::classic::medial_gap::PreparedMedialGapObject],
) -> Vec<usize> {
    objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .flat_map(|surface| {
            std::iter::once(surface.appended.collections.as_ptr() as usize).chain(
                surface.appended.collections.iter().flat_map(|collection| {
                    std::iter::once(collection.entities.as_ptr() as usize)
                        .chain(collection.entities.iter().flat_map(loop_path_allocations))
                }),
            )
        })
        .collect()
}

fn loop_path_allocations(entity: &ExtrusionEntity) -> impl Iterator<Item = usize> + '_ {
    let paths = match entity {
        ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
        ExtrusionEntity::MultiPath(_) => panic!("classic collections keep loop entities"),
    };
    std::iter::once(paths.as_ptr() as usize).chain(
        paths
            .iter()
            .map(|path| path.polyline.points.as_ptr() as usize),
    )
}
