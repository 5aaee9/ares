use crate::{
    SliceError,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        incomplete_sink,
        perimeters::{
            classic::{
                entity_collections::ExtrusionEntity,
                gap_domain,
                perimeter_append::{
                    PreparedPerimeterAppendObject, PreparedPostClassicPerimeterAppend,
                },
            },
            prepare_post_classic_gap_domain, prepare_post_classic_perimeter_append,
        },
    },
};

use super::super::super::super::support::ksr_project;
#[test]
fn task22o11_finish_retains_boxed_o5_and_nested_o10_allocations() {
    let appended = prepare_post_classic_perimeter_append(ksr_project()).unwrap();
    let predecessor = std::ptr::from_ref(appended.predecessor.as_ref());
    let allocations = allocation_pointers(&appended.objects);
    let output = gap_domain::finish(appended).unwrap();
    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(gap_allocation_pointers(&output.objects), allocations);
}

#[test]
fn task22o11_code_level_preparation_reaches_nonempty_pre_medial_domain() {
    let prepared = prepare_post_classic_gap_domain(ksr_project()).unwrap();
    assert!(prepared.objects.iter().any(|object| {
        object.records.iter().flatten().any(|record| {
            record
                .surfaces
                .iter()
                .any(|surface| surface.pre_medial.is_some())
        })
    }));
}

#[test]
fn task22o11_success_and_error_cleanup_are_iterative_on_a_constrained_stack() {
    let mut success = prepare_post_classic_perimeter_append(ksr_project()).unwrap();
    crate::project_slice::tests::deep_cleanup_support::deepen_perimeter_append_trees(&mut success);
    let mut failure = prepare_post_classic_perimeter_append(ksr_project()).unwrap();
    crate::project_slice::tests::deep_cleanup_support::deepen_perimeter_append_trees(&mut failure);
    inject_invalid_gap(&mut failure);

    std::thread::Builder::new()
        .stack_size(crate::project_slice::CONSTRAINED_TEST_STACK_SIZE)
        .spawn(move || {
            let output = gap_domain::finish(success).unwrap();
            for object in output.objects {
                incomplete_sink::consume_gap_domain_object(object);
            }
            incomplete_sink::consume_boxed_post_classic_traversal(output.predecessor);
            assert!(matches!(
                gap_domain::finish(failure),
                Err(SliceError::InvalidInput(message))
                    if message == "Classic gap-domain geometry is outside the supported Clipper range"
            ));
        })
        .unwrap()
        .join()
        .unwrap();
}

fn inject_invalid_gap(prepared: &mut PreparedPostClassicPerimeterAppend) {
    let high = i64::MAX - 1_000_000;
    prepared.predecessor.objects[0]
        .predecessor
        .predecessor
        .records
        .iter_mut()
        .flatten()
        .flat_map(|record| &mut record.surfaces)
        .next()
        .unwrap()
        .gaps = vec![ExPolygon::new(
        Polygon::new(vec![
            Point::new(high, 0),
            Point::new(i64::MAX, 0),
            Point::new(i64::MAX, 1_000_000),
            Point::new(high, 1_000_000),
        ]),
        Vec::new(),
    )];
}

#[derive(Debug, Eq, PartialEq)]
struct AllocationPointers {
    collection_buffers: Vec<usize>,
    entity_buffers: Vec<usize>,
    path_buffers: Vec<usize>,
    point_buffers: Vec<usize>,
}

fn allocation_pointers(objects: &[PreparedPerimeterAppendObject]) -> AllocationPointers {
    let collections = objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .map(|surface| surface.appended.collections.as_slice());
    collect_allocation_pointers(collections)
}

fn gap_allocation_pointers(
    objects: &[crate::project_slice::perimeters::classic::gap_domain::PreparedGapDomainObject],
) -> AllocationPointers {
    let collections = objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
        .map(|surface| surface.appended.collections.as_slice());
    collect_allocation_pointers(collections)
}

fn collect_allocation_pointers<'a>(
    collections: impl Iterator<
        Item = &'a [
            crate::project_slice::perimeters::classic::entity_collections::ExtrusionEntityCollection
        ],
    >,
) -> AllocationPointers {
    let mut pointers = AllocationPointers {
        collection_buffers: Vec::new(),
        entity_buffers: Vec::new(),
        path_buffers: Vec::new(),
        point_buffers: Vec::new(),
    };
    for collections in collections {
        pointers
            .collection_buffers
            .push(collections.as_ptr() as usize);
        for collection in collections {
            collect_entity_allocations(&mut pointers, collection);
        }
    }
    pointers
}

fn collect_entity_allocations(
    pointers: &mut AllocationPointers,
    collection: &crate::project_slice::perimeters::classic::entity_collections::ExtrusionEntityCollection,
) {
    pointers
        .entity_buffers
        .push(collection.entities.as_ptr() as usize);
    for entity in &collection.entities {
        let paths = match entity {
            ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
            ExtrusionEntity::MultiPath(_) => panic!("classic collections keep loop entities"),
        };
        pointers.path_buffers.push(paths.as_ptr() as usize);
        pointers.point_buffers.extend(
            paths
                .iter()
                .map(|path| path.polyline.points.as_ptr() as usize),
        );
    }
}
