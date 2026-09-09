use ares_core::{GenerationMetadata, slice_project};
use std::{fs, path::Path};

mod strict;

/// Per-object extrusion XY bounds over extrusion moves (G1 with E) between
/// the `; printing object` markers, keyed by object name. Wipe segments are
/// skipped: upstream flushes the previous object's retract wipe inside the
/// next object's label block (`GCode.cpp` object-start label queuing).
fn object_extrusion_bounds(
    gcode: &str,
) -> std::collections::BTreeMap<String, (f64, f64, f64, f64)> {
    let mut bounds = std::collections::BTreeMap::new();
    let mut current: Option<String> = None;
    let mut in_wipe = false;
    for line in gcode.lines() {
        if line.contains("WIPE_START") {
            in_wipe = true;
            continue;
        }
        if line.contains("WIPE_END") {
            in_wipe = false;
            continue;
        }
        if in_wipe {
            continue;
        }
        if let Some(name) = line.strip_prefix("; printing object ") {
            current = Some(
                name.split_once(" id:")
                    .map_or(name.to_owned(), |(name, _)| name.to_owned()),
            );
            continue;
        }
        if line.starts_with("; stop printing object ") {
            current = None;
            continue;
        }
        if !line.starts_with("G1 X") {
            continue;
        }
        let Some(name) = &current else { continue };
        let (x, y, has_e) = line
            .split_ascii_whitespace()
            .skip(1)
            .filter_map(|word| {
                let value = word[1..].parse::<f64>().ok()?;
                Some((word.as_bytes()[0], value))
            })
            .fold((0.0, 0.0, false), |(x, y, e), (axis, value)| match axis {
                b'X' => (value, y, e),
                b'Y' => (x, value, e),
                b'E' => (x, y, true),
                _ => (x, y, e),
            });
        if !has_e {
            continue;
        }
        let entry = bounds.entry(name.clone()).or_insert((x, y, x, y));
        entry.0 = entry.0.min(x);
        entry.1 = entry.1.min(y);
        entry.2 = entry.2.max(x);
        entry.3 = entry.3.max(y);
    }
    bounds
}

#[tokio::test]
async fn process_arachne_default_prisms_plate_layout() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/parity/arachne");
    let project = fs::read(directory.join("default-prisms.3mf")).unwrap();
    let reference =
        String::from_utf8(fs::read(directory.join("default-prisms.orca.gcode")).unwrap()).unwrap();
    let metadata = GenerationMetadata::new_local(2026, 9, 8, 0, 0, 0).unwrap();
    let actual =
        String::from_utf8(slice_project(project, metadata).await.expect("ARES_ERROR")).unwrap();

    // Plate-level layer count (`GCode.cpp:2513-2527` merges every object's
    // layer print_z set; per-object emission must not sum to 36).
    assert!(
        actual
            .lines()
            .any(|line| line == "; total layer number: 12"),
        "total layer number must count plate layers, not per-object layers"
    );

    // One terse width block per print region (`GCode.cpp:2676-2693` loops
    // `print.num_print_regions()`; one region per object model part here).
    let width_lines = actual
        .lines()
        .filter(|line| line.contains(" extrusion width = "))
        .count();
    let reference_width_lines = reference
        .lines()
        .filter(|line| line.contains(" extrusion width = "))
        .count();
    assert_eq!(
        width_lines, reference_width_lines,
        "extrusion-width block must be emitted once per object region"
    );

    // Build-item transforms place each object at its own instance offset
    // (`bbs_3mf.cpp:3554-3560` applies build-item transforms; emission
    // re-origins per print object like `GCode.cpp:5380` `set_origin`).
    let actual_bounds = object_extrusion_bounds(&actual);
    let reference_bounds = object_extrusion_bounds(&reference);
    assert_eq!(actual_bounds.len(), 3, "all three objects must be printed");
    for (
        name,
        [
            expected_min_x,
            expected_min_y,
            expected_max_x,
            expected_max_y,
        ],
    ) in reference_bounds
        .iter()
        .map(|(name, bounds)| (name, [bounds.0, bounds.1, bounds.2, bounds.3]))
    {
        let [actual_min_x, actual_min_y, actual_max_x, actual_max_y] = actual_bounds
            .get(name)
            .map(|bounds| [bounds.0, bounds.1, bounds.2, bounds.3])
            .unwrap_or_else(|| panic!("object {name} missing from actual output"));
        for (label, actual, expected) in [
            ("min_x", actual_min_x, expected_min_x),
            ("min_y", actual_min_y, expected_min_y),
            ("max_x", actual_max_x, expected_max_x),
            ("max_y", actual_max_y, expected_max_y),
        ] {
            assert!(
                (actual - expected).abs() <= 0.0015,
                "object {name} {label}: actual {actual} vs reference {expected}"
            );
        }
    }
}

#[tokio::test]
async fn process_arachne_actual_default_prisms_full_output() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/parity/arachne");
    let project = fs::read(directory.join("default-prisms.3mf")).unwrap();
    let reference = fs::read(directory.join("default-prisms.orca.gcode")).unwrap();
    let metadata = GenerationMetadata::new_local(2026, 9, 8, 0, 0, 0).unwrap();
    let actual = slice_project(project, metadata).await.expect("ARES_ERROR");
    strict::compare(&reference, &actual).unwrap();
}
