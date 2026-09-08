use crate::{
    geometry::ExPolygon,
    project_slice::perimeters::{
        classic::gap_extrusion::GapFillEntity, layer_region::PreparedPostLayerRegionPerimeters,
        prepare_post_layer_region_perimeters,
    },
};

use super::super::super::support::KsrArchive;

const CONFIG: &str = "Metadata/project_settings.config";

#[test]
fn task22o16_wall_loop_output_changes_from_typed_3mf_option() {
    let baseline = prepare_post_layer_region_perimeters(&KsrArchive::new().bytes()).unwrap();
    let baseline = perimeter_shape(&baseline);
    assert!(baseline.iter().any(|entry| entry.0 > 0));

    let mut archive = KsrArchive::new();
    archive.replace_unique(CONFIG, "\"wall_loops\": \"2\"", "\"wall_loops\": \"0\"");
    let mutated = prepare_post_layer_region_perimeters(&archive.bytes()).unwrap();
    assert_ne!(perimeter_shape(&mutated), baseline);
}

#[test]
fn task22o16_gap_output_changes_from_typed_3mf_option() {
    let baseline = prepare_post_layer_region_perimeters(&KsrArchive::new().bytes()).unwrap();
    let baseline = thin_fill_shape(&baseline);
    assert!(!baseline.is_empty());

    let mut archive = KsrArchive::new();
    archive.replace_unique(
        CONFIG,
        "\"gap_infill_speed\": \"250\"",
        "\"gap_infill_speed\": \"0\"",
    );
    let mutated = prepare_post_layer_region_perimeters(&archive.bytes()).unwrap();
    assert_ne!(thin_fill_shape(&mutated), baseline);
}

#[test]
fn task22o16_ordinary_overlap_changes_fill_geometry_from_typed_3mf_option() {
    assert_overlap_changes(
        "\"infill_wall_overlap\": \"15%\"",
        "\"infill_wall_overlap\": \"5%\"",
    );
}

#[test]
fn task22o16_top_overlap_changes_fill_geometry_from_typed_3mf_option() {
    assert_overlap_changes(
        "\"top_bottom_infill_wall_overlap\": \"25%\"",
        "\"top_bottom_infill_wall_overlap\": \"5%\"",
    );
}

fn assert_overlap_changes(from: &str, to: &str) {
    let baseline = prepare_post_layer_region_perimeters(&KsrArchive::new().bytes()).unwrap();
    let baseline = fill_checksum(&baseline);
    let mut archive = KsrArchive::new();
    archive.replace_unique(CONFIG, from, to);
    let mutated = prepare_post_layer_region_perimeters(&archive.bytes()).unwrap();
    assert_ne!(fill_checksum(&mutated), baseline);
}

fn perimeter_shape(prepared: &PreparedPostLayerRegionPerimeters) -> Vec<(usize, Vec<i32>)> {
    prepared
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.perimeters)
        .map(|collection| {
            (
                collection.entities.len(),
                collection
                    .entities
                    .iter()
                    .map(|entity| entity.inset_idx())
                    .collect(),
            )
        })
        .collect()
}

fn thin_fill_shape(prepared: &PreparedPostLayerRegionPerimeters) -> Vec<(u8, usize, i64)> {
    prepared
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.thin_fills)
        .map(|entity| match entity {
            GapFillEntity::Path(path) => (1, 1, path.polyline.points[0].x),
            GapFillEntity::Loop(paths) => (2, paths.len(), paths[0].polyline.points[0].x),
        })
        .collect()
}

fn fill_checksum(prepared: &PreparedPostLayerRegionPerimeters) -> i128 {
    let mut checksum = 0_i128;
    for record in prepared
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
    {
        mix(&mut checksum, record.fill_surfaces.len() as i128);
        for surface in &record.fill_surfaces {
            let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
            mix(&mut checksum, kind as i128);
            mix(&mut checksum, i128::from(thickness.to_bits()));
            mix(&mut checksum, i128::from(layers));
            mix(&mut checksum, i128::from(angle.to_bits()));
            mix(&mut checksum, i128::from(extra));
            checksum_expolygon(&mut checksum, expolygon);
        }
        for expolygon in &record.fill_no_overlap_expolygons {
            checksum_expolygon(&mut checksum, expolygon);
        }
    }
    checksum
}

fn checksum_expolygon(checksum: &mut i128, expolygon: &ExPolygon) {
    mix(checksum, expolygon.contour().points().len() as i128);
    for point in expolygon.contour().points() {
        mix(checksum, i128::from(point.x()));
        mix(checksum, i128::from(point.y()));
    }
    mix(checksum, expolygon.holes().len() as i128);
    for hole in expolygon.holes() {
        for point in hole.points() {
            mix(checksum, i128::from(point.x()));
            mix(checksum, i128::from(point.y()));
        }
    }
}

fn mix(checksum: &mut i128, value: i128) {
    *checksum = checksum.wrapping_mul(257).wrapping_add(value);
}
