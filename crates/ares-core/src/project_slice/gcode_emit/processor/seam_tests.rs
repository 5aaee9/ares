use super::{ProcessorLimits, process};
use crate::options::GCodeFlavor;

fn replay(name: &str, reference: &str) {
    // Invert only run_post_process's generated progress and duration fields.
    // Coordinates, roles, F-only commands, M204 and all other bytes are retained.
    let mut first_progress = true;
    let mut input = String::new();
    for line in reference.lines() {
        if line.starts_with("M73 P") && line != "M73 P100 R0" {
            if first_progress {
                input.push_str("M73 P0 R0\n");
                first_progress = false;
            }
            continue;
        }
        if line.starts_with("; estimated printing time (normal mode) =") {
            input.push_str("; estimated printing time (normal mode) = 0s\n");
        } else if line.starts_with("; estimated first layer printing time (normal mode) =") {
            input.push_str("; estimated first layer printing time (normal mode) = 0s\n");
        } else {
            input.push_str(line);
            input.push('\n');
        }
    }
    // Effective config is embedded in the captured actual-AppImage output.
    let limits = ProcessorLimits {
        print_acceleration: 1100.0,
        retract_acceleration: 1100.0,
        travel_acceleration: 1100.0,
        gcode_flavor: GCodeFlavor::MarlinFirmware,
        bbl_printer: false,
        junction_deviation: 0.01,
    };
    let actual = process(input.as_bytes().to_vec(), true, 0.0, 0.0, limits);
    let expected = reference.as_bytes();
    // Optional test-only artifact retention never alters the assertion.
    if let Some(directory) = std::env::var_os("ARES_SEAM_TEST_ARTIFACTS") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join(format!("{name}.input.gcode")), input).unwrap();
        std::fs::write(directory.join(format!("{name}.actual.gcode")), &actual).unwrap();
        std::fs::write(directory.join(format!("{name}.expected.gcode")), expected).unwrap();
    }
    assert!(
        actual == expected,
        "complete processor replay differs: bytes {}/{}, first mismatch {:?}",
        actual.len(),
        expected.len(),
        actual.iter().zip(expected).position(|(a, b)| a != b)
    );
}

#[test]
fn closed_external_wipe_off_matches_actual_orca_output() {
    replay(
        "wipe-off",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/processor_seam_cache/wipe-off.orca.gcode"
        )),
    );
}

#[test]
fn closed_external_inward_before_m204_matches_actual_orca_output() {
    replay(
        "solid-rectilinear",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/processor_seam_cache/solid-rectilinear.orca.gcode"
        )),
    );
}

#[test]
fn anchor_matches_actual_orca_output() {
    replay(
        "anchor",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/processor_seam_cache/anchor.orca.gcode"
        )),
    );
}

#[test]
fn two_walls_matches_actual_orca_output() {
    replay(
        "two-walls",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/processor_seam_cache/two-walls.orca.gcode"
        )),
    );
}

#[test]
fn open_closed_roles_and_first_moving_commands_match_actual_orca_output() {
    replay(
        "classification",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/processor_seam_cache/classification.orca.gcode"
        )),
    );
}
