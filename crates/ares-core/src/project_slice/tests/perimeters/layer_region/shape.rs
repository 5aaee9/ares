use crate::project_slice::perimeters::{
    classic::infill_boundary::PreparedInfillBoundaryObject, layer_region::materialize_object,
    prepare_post_classic_infill_boundary, prepare_post_layer_region_perimeters,
};

use super::super::super::support::ksr_project;

#[test]
fn task22o16_preserves_object_record_and_none_slot_order() {
    let source = prepare_post_classic_infill_boundary(ksr_project()).unwrap();
    let mut records = source
        .objects
        .into_iter()
        .flat_map(|object| object.records)
        .flatten();
    let first = records.next().unwrap();
    let second = records.next().unwrap();
    let third = records.next().unwrap();
    let objects = [
        PreparedInfillBoundaryObject {
            records: vec![None, Some(first), None, Some(second), None],
        },
        PreparedInfillBoundaryObject {
            records: vec![Some(third), None],
        },
    ]
    .map(materialize_object);

    assert_eq!(
        objects.map(|object| {
            object
                .records
                .iter()
                .map(Option::is_some)
                .collect::<Vec<_>>()
        }),
        [vec![false, true, false, true, false], vec![true, false]]
    );
}

#[test]
fn task22o16_whole_project_keeps_every_trusted_one_region_alignment() {
    let output = prepare_post_layer_region_perimeters(ksr_project()).unwrap();
    let classic = output.predecessor.as_classic();
    assert_eq!(output.objects.len(), classic.objects.len());
    for (object, traversal) in output.objects.iter().zip(&classic.objects) {
        let input_object = &traversal
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        assert_eq!(object.records.len(), input_object.records.len());
        let identity = input_object.identity();
        for (record, input) in object.records.iter().zip(&input_object.records) {
            assert_eq!(record.is_some(), input.is_some());
            if let Some(input) = input {
                assert_eq!((input.source_object_index, input.transform_index), identity);
                assert_eq!(input.compatible_region_ids, [input.region_id]);
                assert_eq!(input.current.region_index, input.region_id);
            }
        }
    }
}
