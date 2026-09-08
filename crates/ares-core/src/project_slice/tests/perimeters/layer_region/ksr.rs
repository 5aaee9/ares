use crate::{
    geometry::ExPolygon,
    project_slice::perimeters::{
        classic::{
            chained_loops::ExtrusionLoopRole,
            gap_extrusion::GapFillEntity,
            materialize::{ExtrusionPath, ExtrusionRole},
        },
        layer_region::{PreparedLayerRegionPerimeterRecord, PreparedPostLayerRegionPerimeters},
        prepare_post_layer_region_perimeters,
    },
};

use super::super::super::support::KsrArchive;

const OBJECT_BEGIN: i128 = 0x01_4f424a;
const OBJECT_END: i128 = 0x02_4f424a;
const SLOT_BEGIN: i128 = 0x03_534c54;
const SLOT_END: i128 = 0x04_534c54;
const RECORD_BEGIN: i128 = 0x05_524543;
const RECORD_END: i128 = 0x06_524543;
const PERIMETERS: i128 = 0x07_504552;
const COLLECTION: i128 = 0x08_434f4c;
const LOOP: i128 = 0x09_4c4f50;
const THIN_FILLS: i128 = 0x0a_54484e;
const GAP_PATH: i128 = 0x0b_475054;
const GAP_LOOP: i128 = 0x0c_474c50;
const PATH: i128 = 0x0d_504154;
const FILL_SURFACES: i128 = 0x0e_535552;
const FILL_EXPOLYGONS: i128 = 0x0f_464558;
const NO_OVERLAP: i128 = 0x10_4e4f4f;
const EXPOLYGON: i128 = 0x11_455850;

#[test]
fn task22o16_ksr_perimeter_outputs_are_populated_and_repeatable() {
    let first = prepare_post_layer_region_perimeters(&KsrArchive::new().bytes()).unwrap();
    let second = prepare_post_layer_region_perimeters(&KsrArchive::new().bytes()).unwrap();
    let first_snapshot = (checksum(&first), totals(&first));
    let second_snapshot = (checksum(&second), totals(&second));
    assert_eq!(first_snapshot, second_snapshot);
    assert!(first_snapshot.1[2..].iter().all(|total| *total > 0));
}

pub(in crate::project_slice::tests) fn checksum(
    prepared: &PreparedPostLayerRegionPerimeters,
) -> i128 {
    let mut checksum = 0_i128;
    mix(&mut checksum, prepared.objects.len() as i128);
    let classic = prepared.predecessor.as_classic();
    for (object, traversal) in prepared.objects.iter().zip(&classic.objects) {
        mix(&mut checksum, OBJECT_BEGIN);
        let input_object = &traversal
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        let identity = input_object.identity();
        mix(&mut checksum, identity.0 as i128);
        mix(&mut checksum, identity.1 as i128);
        mix(&mut checksum, object.records.len() as i128);
        for (record, input) in object.records.iter().zip(&input_object.records) {
            mix(&mut checksum, SLOT_BEGIN);
            mix(&mut checksum, i128::from(record.is_some()));
            match (record, input) {
                (Some(record), Some(input)) => {
                    mix(&mut checksum, RECORD_BEGIN);
                    checksum_input(&mut checksum, input);
                    checksum_record(&mut checksum, record);
                    mix(&mut checksum, RECORD_END);
                }
                (None, None) => {}
                _ => panic!("O16 KSR record alignment is invariant"),
            }
            mix(&mut checksum, SLOT_END);
        }
        mix(&mut checksum, OBJECT_END);
    }
    checksum
}

fn checksum_input(
    checksum: &mut i128,
    input: &crate::project_slice::perimeters::types::PerimeterInputRecord,
) {
    for value in [
        input.source_object_index,
        input.transform_index,
        input.planned_layer_index,
        input.layer_id,
        input.region_id,
        input.compatible_region_ids[0],
        input.current.region_index,
        input.current.layer_index,
    ] {
        mix(checksum, value as i128);
    }
    checksum_option(checksum, input.lower_layer_index);
    checksum_option(checksum, input.upper_layer_index);
    match input.upper_same_region {
        Some(index) => {
            mix(checksum, 1);
            mix(checksum, index.region_index as i128);
            mix(checksum, index.layer_index as i128);
        }
        None => mix(checksum, 0),
    }
}

fn checksum_record(checksum: &mut i128, record: &PreparedLayerRegionPerimeterRecord) {
    mix(checksum, PERIMETERS);
    mix(checksum, record.perimeters.len() as i128);
    for collection in &record.perimeters {
        mix(checksum, COLLECTION);
        mix(checksum, collection.entities.len() as i128);
        for entity in &collection.entities {
            mix(checksum, LOOP);
            mix(checksum, i128::from(entity.inset_idx));
            mix(
                checksum,
                match entity.extrusion_loop.role {
                    ExtrusionLoopRole::Internal => 1,
                    ExtrusionLoopRole::Default => 2,
                    ExtrusionLoopRole::Hole => 3,
                },
            );
            mix(checksum, entity.extrusion_loop.paths.len() as i128);
            for path in &entity.extrusion_loop.paths {
                checksum_path(checksum, path);
            }
        }
    }
    mix(checksum, THIN_FILLS);
    mix(checksum, record.thin_fills.len() as i128);
    for entity in &record.thin_fills {
        match entity {
            GapFillEntity::Path(path) => {
                mix(checksum, GAP_PATH);
                checksum_path(checksum, path);
            }
            GapFillEntity::Loop(paths) => {
                mix(checksum, GAP_LOOP);
                mix(checksum, paths.len() as i128);
                for path in paths {
                    checksum_path(checksum, path);
                }
            }
        }
    }
    mix(checksum, FILL_SURFACES);
    mix(checksum, record.fill_surfaces.len() as i128);
    for surface in &record.fill_surfaces {
        let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
        mix(checksum, kind as i128);
        mix(checksum, i128::from(thickness.to_bits()));
        mix(checksum, i128::from(layers));
        mix(checksum, i128::from(angle.to_bits()));
        mix(checksum, i128::from(extra));
        checksum_expolygon(checksum, expolygon);
    }
    mix(checksum, FILL_EXPOLYGONS);
    checksum_expolygons(checksum, &record.fill_expolygons);
    mix(checksum, NO_OVERLAP);
    checksum_expolygons(checksum, &record.fill_no_overlap_expolygons);
}

fn checksum_path(checksum: &mut i128, path: &ExtrusionPath) {
    mix(checksum, PATH);
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

fn checksum_expolygons(checksum: &mut i128, expolygons: &[ExPolygon]) {
    mix(checksum, expolygons.len() as i128);
    for expolygon in expolygons {
        checksum_expolygon(checksum, expolygon);
    }
}

fn checksum_expolygon(checksum: &mut i128, expolygon: &ExPolygon) {
    mix(checksum, EXPOLYGON);
    checksum_points(checksum, expolygon.contour().points());
    mix(checksum, expolygon.holes().len() as i128);
    for hole in expolygon.holes() {
        checksum_points(checksum, hole.points());
    }
}

fn checksum_points(checksum: &mut i128, points: &[crate::geometry::Point]) {
    mix(checksum, points.len() as i128);
    for point in points {
        mix(checksum, i128::from(point.x()));
        mix(checksum, i128::from(point.y()));
    }
}

fn checksum_option(checksum: &mut i128, value: Option<usize>) {
    match value {
        Some(value) => {
            mix(checksum, 1);
            mix(checksum, value as i128);
        }
        None => mix(checksum, 0),
    }
}

pub(in crate::project_slice::tests) fn totals(
    prepared: &PreparedPostLayerRegionPerimeters,
) -> [usize; 9] {
    let slots = prepared
        .objects
        .iter()
        .map(|object| object.records.len())
        .sum();
    let records = prepared
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .collect::<Vec<_>>();
    [
        prepared.objects.len(),
        slots,
        records.len(),
        records.iter().map(|record| record.perimeters.len()).sum(),
        records
            .iter()
            .flat_map(|record| &record.perimeters)
            .map(|collection| collection.entities.len())
            .sum(),
        records.iter().map(|record| record.thin_fills.len()).sum(),
        records
            .iter()
            .map(|record| record.fill_surfaces.len())
            .sum(),
        records
            .iter()
            .map(|record| record.fill_expolygons.len())
            .sum(),
        records
            .iter()
            .map(|record| record.fill_no_overlap_expolygons.len())
            .sum(),
    ]
}

fn mix(checksum: &mut i128, value: i128) {
    *checksum = checksum.wrapping_mul(257).wrapping_add(value);
}
