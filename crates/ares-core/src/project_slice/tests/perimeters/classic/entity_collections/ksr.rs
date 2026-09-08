use crate::project_slice::perimeters::{
    classic::{
        chained_loops::ExtrusionLoopRole,
        entity_collections::{ExtrusionEntity, PreparedEntityCollectionRecord},
        materialize::{ExtrusionPath, ExtrusionRole},
    },
    prepare_post_classic_entity_collections,
};

use super::super::super::super::support::{KsrArchive, ksr_project};

#[test]
fn task22o9_ksr_ordered_entity_fields_are_nonempty_and_repeatable() {
    let first = checksum(ksr_project());
    assert_ne!(first, 0);
    assert_eq!(first, checksum(ksr_project()));
}

#[test]
fn task22o9_ksr_typed_clockwise_wall_direction_reorients_collections() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"wall_direction\": \"ccw\"",
        "\"wall_direction\": \"cw\"",
    );
    let project = archive.bytes();
    let clockwise = checksum(&project);
    assert_ne!(clockwise, 0);
    assert_ne!(clockwise, checksum(ksr_project()));
}

#[test]
fn task22o9_ksr_preserves_alignment_roles_depth_and_closed_loops() {
    let prepared = prepare_post_classic_entity_collections(ksr_project()).unwrap();
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    let mut entities = 0;
    let mut roles = [0_usize; 3];
    for (output, traversal) in prepared.objects.iter().zip(&prepared.predecessor.objects) {
        assert_eq!(output.records.len(), traversal.records.len());
        for (output_record, traversal_record) in output.records.iter().zip(&traversal.records) {
            assert_eq!(output_record.is_some(), traversal_record.is_some());
            let (Some(output_record), Some(traversal_record)) = (output_record, traversal_record)
            else {
                continue;
            };
            inspect_record(output_record, traversal_record, &mut entities, &mut roles);
        }
    }
    assert!(entities > 0);
    assert!(roles.iter().all(|count| *count > 0));
}

fn inspect_record(
    output: &PreparedEntityCollectionRecord,
    traversal: &crate::project_slice::perimeters::classic::traversal::ClassicTraversalRecord,
    entity_count: &mut usize,
    roles: &mut [usize; 3],
) {
    assert_eq!(output.surfaces.len(), traversal.surfaces.len());
    for (surface, seeds) in output.surfaces.iter().zip(&traversal.surfaces) {
        assert_eq!(surface.source_index, seeds.source_index);
        inspect_entities(&surface.collection.entities, entity_count, roles);
    }
}

fn inspect_entities(
    entities: &[ExtrusionEntity],
    entity_count: &mut usize,
    roles: &mut [usize; 3],
) {
    for entity in entities {
        let ordered = match entity {
            ExtrusionEntity::Loop(ordered) => ordered,
            ExtrusionEntity::MultiPath(_) => panic!("classic traversal emits loop entities"),
        };
        *entity_count += 1;
        roles[match ordered.extrusion_loop.role {
            ExtrusionLoopRole::Internal => 0,
            ExtrusionLoopRole::Default => 1,
            ExtrusionLoopRole::Hole => 2,
        }] += 1;
        assert!(!ordered.extrusion_loop.paths.is_empty());
        let first = ordered.extrusion_loop.paths[0].polyline.points[0];
        let last_path = ordered.extrusion_loop.paths.last().unwrap();
        assert_eq!(first, *last_path.polyline.points.last().unwrap());
    }
}

fn checksum(project: impl AsRef<[u8]>) -> i128 {
    let prepared = prepare_post_classic_entity_collections(project.as_ref()).unwrap();
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

fn accumulate_record(record: &PreparedEntityCollectionRecord, checksum: &mut i128) {
    mix(checksum, record.surfaces.len() as i128);
    for surface in &record.surfaces {
        mix(checksum, surface.source_index as i128);
        accumulate_entities(&surface.collection.entities, checksum);
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
