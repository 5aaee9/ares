use super::{ProcessorLimits, process};
use std::{fs, path::PathBuf};

fn reference(case: &str) -> String {
    let root = PathBuf::from(std::env::var_os("ARES_ENVELOPE_FIXTURES").unwrap());
    fs::read_to_string(root.join(case).join("orca.gcode")).unwrap()
}

fn setting(reference: &str, key: &str) -> f64 {
    let prefix = format!("; {key} = ");
    reference
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap()
        .split(',')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

fn replay(name: &str, reference: &str) -> Vec<u8> {
    let mut first = true;
    let mut input = String::new();
    for line in reference.lines() {
        if line.starts_with("M73 P") && line != "M73 P100 R0" {
            if first {
                input.push_str("M73 P0 R0\n");
                first = false;
            }
        } else if line.starts_with("; estimated printing time (normal mode) =") {
            input.push_str("; estimated printing time (normal mode) = 0s\n");
        } else if line.starts_with("; estimated first layer printing time (normal mode) =") {
            input.push_str("; estimated first layer printing time (normal mode) = 0s\n");
        } else {
            input.push_str(line);
            input.push('\n');
        }
    }
    let flavor = reference
        .lines()
        .find_map(|line| line.strip_prefix("; gcode_flavor = "))
        .unwrap();
    let flavor = serde_json::from_value(serde_json::Value::String(flavor.to_owned())).unwrap();
    let mut config = serde_json::Map::new();
    for key in crate::MachineEnvelopeOptions::DECLARATION_ORDER {
        let prefix = format!("; {key} = ");
        if let Some(value) = reference
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
        {
            let value = if key.starts_with("machine_max_") || key.starts_with("machine_min_") {
                serde_json::Value::Array(value.split(',').map(|v| v.into()).collect())
            } else {
                value.into()
            };
            config.insert(key.to_owned(), value);
        }
    }
    let machine = serde_json::from_value(config.into()).unwrap();
    let limits = ProcessorLimits::from_config(&machine, flavor, false);
    let actual = process(
        input.as_bytes().to_vec(),
        true,
        setting(reference, "machine_load_filament_time"),
        0.0,
        limits,
    );
    if let Some(root) = std::env::var_os("ARES_ENVELOPE_OUTPUT") {
        let root = PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(format!("{name}.input.gcode")), input).unwrap();
        fs::write(root.join(format!("{name}.actual.gcode")), &actual).unwrap();
        fs::write(root.join(format!("{name}.expected.gcode")), reference).unwrap();
    }
    actual
}

fn strict(name: &str, reference: &str) {
    let actual = replay(name, reference);
    assert!(
        actual == reference.as_bytes(),
        "{name}: complete output differs, first byte {:?}",
        actual
            .iter()
            .zip(reference.bytes())
            .position(|(a, b)| *a != b)
    );
}

#[test]
fn positive_actual_orca_replay() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/processor_seam_cache/anchor.orca.gcode");
    strict("anchor", &fs::read_to_string(path).unwrap());
}

#[test]
#[ignore = "requires immutable wave8 actual-AppImage artifacts; complete parity diagnostic"]
fn afinia_complete_actual_orca_replay() {
    strict("afinia", &reference("case-oKLMEh"));
}

#[test]
#[ignore = "requires immutable wave8 actual-AppImage artifacts; complete parity diagnostic"]
fn artillery_complete_actual_orca_replay() {
    strict("artillery", &reference("case-TQeCzN"));
}
