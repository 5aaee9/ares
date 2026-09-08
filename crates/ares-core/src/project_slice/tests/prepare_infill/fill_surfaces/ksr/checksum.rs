use crate::{
    geometry::ExPolygon,
    project_slice::{
        perimeters::{
            classic::{
                chained_loops::ExtrusionLoopRole,
                entity_collections::ExtrusionEntity,
                gap_extrusion::GapFillEntity,
                materialize::{ExtrusionPath, ExtrusionRole},
                traversal::PreparedPostClassicTraversal,
            },
            types::PerimeterInputRecord,
        },
        prepare_infill::surface_type_detection,
        region_slices::RegionSurface,
    },
};

pub(in crate::project_slice::tests::prepare_infill) fn checksum(
    predecessor: &PreparedPostClassicTraversal,
    objects: &[surface_type_detection::PreparedSurfaceTypeObject],
) -> i128 {
    let mut checksum = -169_716_507_603_417_685_621_692_788_651_154_411_580_i128;
    mix(&mut checksum, objects.len() as i128);
    for (object, traversal) in objects.iter().zip(&predecessor.objects) {
        mix(&mut checksum, 0x01_4f424a);
        let input = &traversal
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        let identity = input.identity();
        mix(&mut checksum, identity.0 as i128);
        mix(&mut checksum, identity.1 as i128);
        mix(&mut checksum, object.records.len() as i128);
        for (record, input) in object.records.iter().zip(&input.records) {
            mix(&mut checksum, 0x02_534c54);
            mix(&mut checksum, i128::from(record.is_some()));
            match (record, input) {
                (Some(record), Some(input)) => checksum_record(&mut checksum, record, input),
                (None, None) => {}
                _ => panic!("O18 KSR records remain aligned"),
            }
            mix(&mut checksum, 0x03_534c54);
        }
        mix(&mut checksum, 0x04_4f424a);
    }
    checksum
}

fn checksum_record(
    checksum: &mut i128,
    record: &surface_type_detection::types::PreparedSurfaceTypeRecord,
    input: &PerimeterInputRecord,
) {
    for value in [
        input.source_object_index,
        input.transform_index,
        input.planned_layer_index,
        input.layer_id,
        input.region_id,
        record.perimeters.len(),
        record.thin_fills.len(),
        record.fill_expolygons.len(),
        record.fill_no_overlap_expolygons.len(),
    ] {
        mix(checksum, value as i128);
    }
    checksum_surfaces(checksum, &record.slices);
    checksum_surfaces(checksum, &record.fill_surfaces);
}

fn checksum_surfaces(checksum: &mut i128, surfaces: &[RegionSurface]) {
    mix(checksum, surfaces.len() as i128);
    for surface in surfaces {
        let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
        for value in [
            kind as i128,
            i128::from(thickness.to_bits()),
            i128::from(layers),
            i128::from(angle.to_bits()),
            i128::from(extra),
        ] {
            mix(checksum, value);
        }
        checksum_expolygon(checksum, expolygon);
    }
}

pub(in crate::project_slice::tests) fn unrelated_checksum(
    predecessor: &PreparedPostClassicTraversal,
    objects: &[surface_type_detection::PreparedSurfaceTypeObject],
) -> i128 {
    let mut checksum = 0_i128;
    mix(&mut checksum, objects.len() as i128);
    for (object, traversal) in objects.iter().zip(&predecessor.objects) {
        let input_object = &traversal
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        mix(&mut checksum, input_object.identity().0 as i128);
        mix(&mut checksum, input_object.identity().1 as i128);
        mix(&mut checksum, object.records.len() as i128);
        for (record, input) in object.records.iter().zip(&input_object.records) {
            mix(&mut checksum, i128::from(record.is_some()));
            match (record, input) {
                (Some(record), Some(input)) => {
                    checksum_input(&mut checksum, input);
                    checksum_untouched_record(&mut checksum, record);
                }
                (None, None) => {}
                _ => panic!("O18 unrelated checksum requires aligned records"),
            }
        }
    }
    checksum
}

fn checksum_input(checksum: &mut i128, input: &PerimeterInputRecord) {
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
    for value in [input.lower_layer_index, input.upper_layer_index] {
        match value {
            Some(value) => {
                mix(checksum, 1);
                mix(checksum, value as i128);
            }
            None => mix(checksum, 0),
        }
    }
    match input.upper_same_region {
        Some(index) => {
            mix(checksum, 1);
            mix(checksum, index.region_index as i128);
            mix(checksum, index.layer_index as i128);
        }
        None => mix(checksum, 0),
    }
}

fn checksum_untouched_record(
    checksum: &mut i128,
    record: &surface_type_detection::types::PreparedSurfaceTypeRecord,
) {
    mix(checksum, record.perimeters.len() as i128);
    for collection in &record.perimeters {
        mix(checksum, collection.entities.len() as i128);
        for entity in &collection.entities {
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
    }
    mix(checksum, record.thin_fills.len() as i128);
    for entity in &record.thin_fills {
        match entity {
            GapFillEntity::Path(path) => {
                mix(checksum, 1);
                checksum_path(checksum, path);
            }
            GapFillEntity::Loop(paths) => {
                mix(checksum, 2);
                mix(checksum, paths.len() as i128);
                for path in paths {
                    checksum_path(checksum, path);
                }
            }
        }
    }
    checksum_surfaces(checksum, &record.slices);
    checksum_surface_payloads(checksum, &record.fill_surfaces);
    checksum_expolygons(checksum, &record.fill_expolygons);
    checksum_expolygons(checksum, &record.fill_no_overlap_expolygons);
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

fn checksum_surface_payloads(checksum: &mut i128, surfaces: &[RegionSurface]) {
    mix(checksum, surfaces.len() as i128);
    for surface in surfaces {
        let (_, expolygon, thickness, layers, angle, extra) = surface.as_parts();
        for value in [
            i128::from(thickness.to_bits()),
            i128::from(layers),
            i128::from(angle.to_bits()),
            i128::from(extra),
        ] {
            mix(checksum, value);
        }
        checksum_expolygon(checksum, expolygon);
    }
}

fn checksum_expolygons(checksum: &mut i128, expolygons: &[ExPolygon]) {
    mix(checksum, expolygons.len() as i128);
    for expolygon in expolygons {
        checksum_expolygon(checksum, expolygon);
    }
}

fn checksum_expolygon(checksum: &mut i128, expolygon: &ExPolygon) {
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

fn mix(checksum: &mut i128, value: i128) {
    *checksum = checksum.wrapping_mul(257).wrapping_add(value);
}
