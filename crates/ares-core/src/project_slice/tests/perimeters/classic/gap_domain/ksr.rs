use crate::{
    ProcessBrimType,
    project_slice::perimeters::{
        classic::{
            chained_loops::ExtrusionLoopRole,
            entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
            gap_domain,
            materialize::{ExtrusionPath, ExtrusionRole},
            perimeter_append::{
                InactiveOuterBrimReordering, InactiveOverhangReorientation,
                InactivePostCollectionBranches, InactiveWallReordering,
            },
        },
        prepare_post_classic_gap_domain, prepare_post_classic_perimeter_append,
    },
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o11_ksr_gap_domain_structure_is_nonempty_and_deterministic() {
    let first = gap_checksum(
        &prepare_post_classic_gap_domain(ksr_project())
            .unwrap()
            .objects,
    );
    let second = gap_checksum(
        &prepare_post_classic_gap_domain(ksr_project())
            .unwrap()
            .objects,
    );
    assert_ne!(first, 0);
    assert_eq!(first, second);
}

#[test]
fn task22o11_ksr_moving_o10_collections_preserves_their_checksum() {
    let appended = prepare_post_classic_perimeter_append(ksr_project()).unwrap();
    let before = collection_checksum_append(&appended.objects);
    let output = gap_domain::finish(appended).unwrap();
    assert_eq!(collection_checksum_gap(&output.objects), before);
}

fn gap_checksum(
    objects: &[crate::project_slice::perimeters::classic::gap_domain::PreparedGapDomainObject],
) -> i128 {
    let mut checksum = 0_i128;
    mix(&mut checksum, objects.len() as i128);
    for object in objects {
        mix(&mut checksum, object.records.len() as i128);
        for record in &object.records {
            mix(&mut checksum, i128::from(record.is_some()));
            if let Some(record) = record {
                checksum_gap_record(&mut checksum, record);
            }
        }
    }
    checksum
}

fn checksum_gap_record(
    checksum: &mut i128,
    record: &crate::project_slice::perimeters::classic::gap_domain::PreparedGapDomainRecord,
) {
    mix(checksum, record.surfaces.len() as i128);
    for surface in &record.surfaces {
        mix(checksum, surface.source_index as i128);
        let Some(domain) = &surface.pre_medial else {
            mix(checksum, 0);
            continue;
        };
        mix(checksum, 1);
        mix(checksum, i128::from(domain.min.to_bits()));
        mix(checksum, i128::from(domain.max.to_bits()));
        mix(checksum, domain.expolygons.len() as i128);
        for expolygon in &domain.expolygons {
            checksum_polygon(checksum, expolygon.contour());
            mix(checksum, expolygon.holes().len() as i128);
            for hole in expolygon.holes() {
                checksum_polygon(checksum, hole);
            }
        }
    }
}

fn collection_checksum_append(
    objects: &[crate::project_slice::perimeters::classic::perimeter_append::PreparedPerimeterAppendObject],
) -> i128 {
    let mut checksum = 0_i128;
    mix(&mut checksum, objects.len() as i128);
    for object in objects {
        mix(&mut checksum, object.records.len() as i128);
        for record in &object.records {
            mix(&mut checksum, i128::from(record.is_some()));
            if let Some(record) = record {
                checksum_append_record(&mut checksum, record);
            }
        }
    }
    checksum
}

fn collection_checksum_gap(
    objects: &[crate::project_slice::perimeters::classic::gap_domain::PreparedGapDomainObject],
) -> i128 {
    let mut checksum = 0_i128;
    mix(&mut checksum, objects.len() as i128);
    for object in objects {
        mix(&mut checksum, object.records.len() as i128);
        for record in &object.records {
            mix(&mut checksum, i128::from(record.is_some()));
            if let Some(record) = record {
                checksum_gap_collection_record(&mut checksum, record);
            }
        }
    }
    checksum
}

fn checksum_append_record(
    checksum: &mut i128,
    record: &crate::project_slice::perimeters::classic::perimeter_append::PreparedPerimeterAppendRecord,
) {
    mix(checksum, record.surfaces.len() as i128);
    for surface in &record.surfaces {
        checksum_collection_surface(
            checksum,
            surface.source_index,
            surface.inactive,
            &surface.appended.collections,
        );
    }
}

fn checksum_gap_collection_record(
    checksum: &mut i128,
    record: &crate::project_slice::perimeters::classic::gap_domain::PreparedGapDomainRecord,
) {
    mix(checksum, record.surfaces.len() as i128);
    for surface in &record.surfaces {
        checksum_collection_surface(
            checksum,
            surface.source_index,
            surface.inactive,
            &surface.appended.collections,
        );
    }
}

fn checksum_collection_surface(
    checksum: &mut i128,
    source_index: usize,
    inactive: InactivePostCollectionBranches,
    collections: &[ExtrusionEntityCollection],
) {
    mix(checksum, source_index as i128);
    checksum_inactive(checksum, inactive);
    mix(checksum, collections.len() as i128);
    for collection in collections {
        mix(checksum, collection.entities.len() as i128);
        for entity in &collection.entities {
            checksum_entity(checksum, entity);
        }
    }
}

fn checksum_polygon(checksum: &mut i128, polygon: &crate::geometry::Polygon) {
    mix(checksum, polygon.points().len() as i128);
    for point in polygon.points() {
        mix(checksum, i128::from(point.x()));
        mix(checksum, i128::from(point.y()));
    }
}

fn checksum_inactive(checksum: &mut i128, inactive: InactivePostCollectionBranches) {
    let InactiveOverhangReorientation::Disabled {
        overhang_reverse_internal_only,
    } = inactive.overhang_reorientation;
    mix(checksum, i128::from(overhang_reverse_internal_only));
    let InactiveWallReordering::InnerOuter { outer_brim } = inactive.wall_reordering;
    checksum_outer_brim(checksum, outer_brim);
}

fn checksum_outer_brim(checksum: &mut i128, reason: InactiveOuterBrimReordering) {
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

fn checksum_entity(checksum: &mut i128, entity: &ExtrusionEntity) {
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
        checksum_path(checksum, path);
    }
}

fn checksum_path(checksum: &mut i128, path: &ExtrusionPath) {
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
