use crate::{
    ProcessBrimType,
    project_slice::perimeters::{
        classic::{
            chained_loops::ExtrusionLoopRole,
            entity_collections::ExtrusionEntity,
            materialize::{ExtrusionPath, ExtrusionRole},
            perimeter_append::{
                InactiveOuterBrimReordering, InactiveOverhangReorientation, InactiveWallReordering,
                PreparedPerimeterAppendRecord,
            },
        },
        prepare_post_classic_entity_collections, prepare_post_classic_perimeter_append,
    },
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o10_ksr_nonempty_append_and_alignment_match_o9() {
    let appended = prepare_post_classic_perimeter_append(ksr_project()).unwrap();
    let source = prepare_post_classic_entity_collections(ksr_project()).unwrap();
    assert_eq!(appended.objects.len(), source.objects.len());
    for (appended_object, source_object) in appended.objects.iter().zip(&source.objects) {
        assert_eq!(appended_object.records.len(), source_object.records.len());
        for (appended_record, source_record) in
            appended_object.records.iter().zip(&source_object.records)
        {
            assert_eq!(appended_record.is_some(), source_record.is_some());
            let (Some(appended_record), Some(source_record)) = (appended_record, source_record)
            else {
                continue;
            };
            assert_record_append(appended_record, source_record);
        }
    }
}

fn assert_record_append(
    appended: &PreparedPerimeterAppendRecord,
    source: &crate::project_slice::perimeters::classic::entity_collections::PreparedEntityCollectionRecord,
) {
    assert_eq!(appended.surfaces.len(), source.surfaces.len());
    for (appended_surface, source_surface) in appended.surfaces.iter().zip(&source.surfaces) {
        assert_eq!(appended_surface.source_index, source_surface.source_index);
        assert_eq!(
            appended_surface.appended.collections.len(),
            usize::from(!source_surface.collection.entities.is_empty())
        );
        if let Some(collection) = appended_surface.appended.collections.first() {
            assert_eq!(collection, &source_surface.collection);
        }
    }
}

#[test]
fn task22o10_ksr_nested_append_provenance_is_nonempty_and_repeatable() {
    let first = checksum();
    assert_ne!(first, 0);
    assert_eq!(first, checksum());
}

fn checksum() -> i128 {
    let prepared = prepare_post_classic_perimeter_append(ksr_project()).unwrap();
    let mut checksum = 0_i128;
    mix(&mut checksum, prepared.objects.len() as i128);
    for object in &prepared.objects {
        mix(&mut checksum, object.records.len() as i128);
        for record in &object.records {
            mix(&mut checksum, i128::from(record.is_some()));
            if let Some(record) = record {
                accumulate_record(record, &mut checksum);
            }
        }
    }
    checksum
}

fn accumulate_record(record: &PreparedPerimeterAppendRecord, checksum: &mut i128) {
    mix(checksum, record.surfaces.len() as i128);
    for surface in &record.surfaces {
        mix(checksum, surface.source_index as i128);
        let InactiveOverhangReorientation::Disabled {
            overhang_reverse_internal_only,
        } = surface.inactive.overhang_reorientation;
        mix(checksum, i128::from(overhang_reverse_internal_only));
        let InactiveWallReordering::InnerOuter { outer_brim } = surface.inactive.wall_reordering;
        accumulate_outer_brim(outer_brim, checksum);
        mix(checksum, surface.appended.collections.len() as i128);
        for collection in &surface.appended.collections {
            accumulate_entities(&collection.entities, checksum);
        }
    }
}

fn accumulate_outer_brim(reason: InactiveOuterBrimReordering, checksum: &mut i128) {
    match reason {
        InactiveOuterBrimReordering::LaterLayer {
            layer_id,
            brim_type,
            brim_width,
        } => {
            mix(checksum, 1);
            mix(checksum, layer_id as i128);
            mix(checksum, brim_type_code(brim_type));
            mix(checksum, i128::from(brim_width.to_bits()));
        }
        InactiveOuterBrimReordering::DifferentBrimType {
            brim_type,
            brim_width,
        } => {
            mix(checksum, 2);
            mix(checksum, brim_type_code(brim_type));
            mix(checksum, i128::from(brim_width.to_bits()));
        }
        InactiveOuterBrimReordering::WidthNotPositive { brim_width } => {
            mix(checksum, 3);
            mix(checksum, i128::from(brim_width.to_bits()));
        }
    }
}

fn accumulate_entities(entities: &[ExtrusionEntity], checksum: &mut i128) {
    mix(checksum, entities.len() as i128);
    for entity in entities {
        let ordered = match entity {
            ExtrusionEntity::Loop(ordered) => ordered,
            ExtrusionEntity::MultiPath(_) => panic!("classic traversal emits loop entities"),
        };
        mix(checksum, i128::from(ordered.inset_idx));
        mix(
            checksum,
            match ordered.extrusion_loop.role {
                ExtrusionLoopRole::Internal => 1,
                ExtrusionLoopRole::Default => 2,
                ExtrusionLoopRole::Hole => 3,
            },
        );
        mix(checksum, ordered.extrusion_loop.paths.len() as i128);
        for path in &ordered.extrusion_loop.paths {
            accumulate_path(path, checksum);
        }
    }
}

fn brim_type_code(brim_type: ProcessBrimType) -> i128 {
    match brim_type {
        ProcessBrimType::AutoBrim => 0,
        ProcessBrimType::BrimEars => 1,
        ProcessBrimType::Painted => 2,
        ProcessBrimType::OuterOnly => 3,
        ProcessBrimType::InnerOnly => 4,
        ProcessBrimType::OuterAndInner => 5,
        ProcessBrimType::NoBrim => 6,
    }
}

fn accumulate_path(path: &ExtrusionPath, checksum: &mut i128) {
    mix(
        checksum,
        match path.role {
            ExtrusionRole::ExternalPerimeter => 1,
            ExtrusionRole::Perimeter => 2,
            ExtrusionRole::OverhangPerimeter => 3,
            ExtrusionRole::GapFill => 4,
            ExtrusionRole::SolidInfill => 5,
            ExtrusionRole::TopSolidInfill => 6,
            ExtrusionRole::BottomSurface => 7,
        },
    );
    mix(checksum, i128::from(path.mm3_per_mm.to_bits()));
    mix(checksum, i128::from(path.width.to_bits()));
    mix(checksum, i128::from(path.height.to_bits()));
    mix(checksum, path.polyline.points.len() as i128);
    for point in &path.polyline.points {
        mix(checksum, i128::from(point.x));
        mix(checksum, i128::from(point.y));
        mix(checksum, i128::from(point.z));
    }
}

fn mix(checksum: &mut i128, value: i128) {
    *checksum = checksum.wrapping_mul(257).wrapping_add(value);
}
