use crate::{SliceError, slice_project};

use super::super::super::support::{KsrArchive, metadata};

#[tokio::test]
async fn fuzzy_skin_reaches_active_classic_output() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"fuzzy_skin\": \"disabled_fuzzy\"",
        "\"fuzzy_skin\": \"external\"",
    );

    let output = slice_project(archive.bytes(), metadata()).await.unwrap();

    assert!(!output.is_empty());
}

#[tokio::test]
async fn task22o1_preflight_rejects_each_activated_deferred_classic_branch() {
    let cases = [
        // Arachne dispatch itself is enabled; the KSR configuration hits
        // the arachne first-slice scope gate (`PerimeterGenerator.cpp:2168-
        // 2224` top-surface regeneration stays deferred).
        (
            "\"wall_generator\": \"classic\"",
            "\"wall_generator\": \"arachne\"",
            "only_one_wall_top",
        ),
        (
            "\"counterbore_hole_bridging\": \"none\"",
            "\"counterbore_hole_bridging\": \"partiallybridge\"",
            "counterbore_hole_bridging",
        ),
    ];

    for (from, to, key) in cases {
        let mut archive = KsrArchive::new();
        archive.replace_unique("Metadata/project_settings.config", from, to);
        assert_eq!(
            slice_project(archive.bytes(), metadata())
                .await
                .unwrap_err(),
            SliceError::UnsupportedProjectFeature(key.to_owned()),
            "{key}"
        );
    }
}

#[tokio::test]
async fn alternate_extra_wall_adds_one_wall_on_the_second_layer() {
    let baseline = slice_project(KsrArchive::new().bytes(), metadata())
        .await
        .unwrap();
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"alternate_extra_wall\": \"0\"",
        "\"alternate_extra_wall\": \"1\"",
    );
    let alternate = slice_project(archive.bytes(), metadata()).await.unwrap();
    let used_filament = |gcode: &[u8]| {
        String::from_utf8_lossy(gcode)
            .lines()
            .find_map(|line| line.strip_prefix("; filament used [mm] = "))
            .unwrap()
            .split(',')
            .next()
            .unwrap()
            .parse::<f64>()
            .unwrap()
    };

    assert!(used_filament(&alternate) > used_filament(&baseline));
}

#[tokio::test]
async fn only_one_wall_first_layer_reduces_first_layer_perimeters() {
    let baseline = slice_project(KsrArchive::new().bytes(), metadata())
        .await
        .unwrap();
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"only_one_wall_first_layer\": \"0\"",
        "\"only_one_wall_first_layer\": \"1\"",
    );

    let reduced = slice_project(archive.bytes(), metadata()).await.unwrap();

    let inner_walls = |gcode: &[u8]| {
        String::from_utf8_lossy(gcode)
            .split("; CHANGE_LAYER")
            .nth(1)
            .unwrap()
            .matches("; FEATURE: Inner wall")
            .count()
    };
    assert!(inner_walls(&reduced) < inner_walls(&baseline));
}

#[tokio::test]
async fn task22o1_preflight_keeps_earlier_raft_gate_precedence() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\t\"raft_layers\": \"0\",",
        "\t\"raft_layers\": \"1\",",
    );
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\t\"wall_generator\": \"classic\",",
        "\t\"wall_generator\": \"arachne\",",
    );
    assert_eq!(
        slice_project(archive.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::UnsupportedProjectFeature("raft_layers".to_owned())
    );
}
