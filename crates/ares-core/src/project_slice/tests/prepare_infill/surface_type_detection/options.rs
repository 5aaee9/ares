use crate::{
    project_slice::{
        perimeters::prepare_post_layer_region_perimeters,
        prepare_infill::surface_type_detection::{
            self, PreparedPostSurfaceTypeDetection, invocations, reset_geometry_hooks,
            reset_invocations,
        },
        region_slices::RegionSurfaceKind,
        tests::perimeters::layer_region::ksr as o16,
    },
    slice_project,
};

use super::super::super::support::{KsrArchive, metadata};

fn kind_counts(prepared: &PreparedPostSurfaceTypeDetection) -> [usize; 4] {
    let mut counts = [0; 4];
    for record in prepared
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
    {
        for surface in record.slices.iter().chain(&record.fill_surfaces) {
            counts[match surface.as_parts().0 {
                RegionSurfaceKind::Top => 0,
                RegionSurfaceKind::Bottom => 1,
                RegionSurfaceKind::BottomBridge => 2,
                RegionSurfaceKind::Internal => 3,
                RegionSurfaceKind::InternalSolid
                | RegionSurfaceKind::InternalBridge
                | RegionSurfaceKind::InternalVoid => {
                    panic!("O17 cannot emit internal solid or void surfaces")
                }
            }] += 1;
        }
    }
    counts
}

fn coordinate_checksum(prepared: &PreparedPostSurfaceTypeDetection) -> i128 {
    prepared
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| record.slices.iter().chain(&record.fill_surfaces))
        .flat_map(|surface| {
            let expolygon = surface.as_parts().1;
            std::iter::once(expolygon.contour()).chain(expolygon.holes())
        })
        .flat_map(|polygon| polygon.points())
        .fold(0_i128, |checksum, point| {
            checksum
                .wrapping_mul(257)
                .wrapping_add(i128::from(point.x()))
                .wrapping_mul(257)
                .wrapping_add(i128::from(point.y()))
        })
}

fn geometry_metadata_checksum(prepared: &PreparedPostSurfaceTypeDetection) -> i128 {
    prepared
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| record.slices.iter().chain(&record.fill_surfaces))
        .fold(0_i128, |mut checksum, surface| {
            let (_, expolygon, thickness, layers, angle, extra) = surface.as_parts();
            for value in [
                i128::from(thickness.to_bits()),
                i128::from(layers),
                i128::from(angle.to_bits()),
                i128::from(extra),
            ] {
                checksum = checksum.wrapping_mul(257).wrapping_add(value);
            }
            for polygon in std::iter::once(expolygon.contour()).chain(expolygon.holes()) {
                for point in polygon.points() {
                    checksum = checksum
                        .wrapping_mul(257)
                        .wrapping_add(i128::from(point.x()))
                        .wrapping_mul(257)
                        .wrapping_add(i128::from(point.y()));
                }
            }
            checksum
        })
}

fn prepare_option_case(archive: KsrArchive) -> (PreparedPostSurfaceTypeDetection, i128) {
    let source = prepare_post_layer_region_perimeters(&archive.bytes()).unwrap();
    let predecessor = std::ptr::from_ref(source.predecessor.as_classic());
    let checksum = o16::checksum(&source);
    let output = surface_type_detection::prepare(source).unwrap();
    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    (output, checksum)
}

#[tokio::test]
async fn task22o17_interface_shells_single_region_reaches_output() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"interface_shells\": \"0\"",
        "\"interface_shells\": \"1\"",
    );
    reset_geometry_hooks();
    reset_invocations();
    let output = slice_project(archive.bytes(), metadata()).await.unwrap();
    assert!(!output.is_empty());
    assert_eq!(invocations(), 1);
}

#[tokio::test]
async fn task22o17_extra_bridge_values_reach_active_output() {
    for token in [
        "external_bridge_only",
        "internal_bridge_only",
        "apply_to_all",
    ] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"enable_extra_bridge_layer\": \"disabled\"",
            &format!("\"enable_extra_bridge_layer\": \"{token}\""),
        );
        assert!(
            !slice_project(archive.bytes(), metadata())
                .await
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn task22o17_support_only_mutations_change_only_the_source_bottom_kind() {
    let (baseline, baseline_o16) = prepare_option_case(KsrArchive::new());
    let mut supported = KsrArchive::new();
    supported.replace_unique(
        "Metadata/project_settings.config",
        "\"enable_support\": \"0\"",
        "\"enable_support\": \"1\"",
    );
    supported.replace_unique(
        "Metadata/project_settings.config",
        "\"support_top_z_distance\": \"0.2\"",
        "\"support_top_z_distance\": \"0\"",
    );
    let (supported, supported_o16) = prepare_option_case(supported);

    let mut unsupported = KsrArchive::new();
    unsupported.replace_unique(
        "Metadata/project_settings.config",
        "\"enable_support\": \"0\"",
        "\"enable_support\": \"1\"",
    );
    unsupported.replace_unique(
        "Metadata/project_settings.config",
        "\"support_top_z_distance\": \"0.2\"",
        "\"support_top_z_distance\": \"0\"",
    );
    unsupported.replace_unique(
        "Metadata/project_settings.config",
        "\"support_critical_regions_only\": \"0\"",
        "\"support_critical_regions_only\": \"1\"",
    );
    let (unsupported, unsupported_o16) = prepare_option_case(unsupported);

    let baseline_counts = kind_counts(&baseline);
    let supported_counts = kind_counts(&supported);
    assert!(baseline_counts[2] > supported_counts[2]);
    assert!(supported_counts[1] > baseline_counts[1]);
    assert_eq!(kind_counts(&unsupported), baseline_counts);
    assert_eq!(
        coordinate_checksum(&baseline),
        coordinate_checksum(&supported)
    );
    assert_eq!(
        coordinate_checksum(&baseline),
        coordinate_checksum(&unsupported)
    );
    assert_eq!(
        geometry_metadata_checksum(&baseline),
        geometry_metadata_checksum(&supported)
    );
    assert_eq!(
        geometry_metadata_checksum(&baseline),
        geometry_metadata_checksum(&unsupported)
    );
    assert_eq!(supported_o16, baseline_o16);
    assert_eq!(unsupported_o16, baseline_o16);
}
