use super::application::{verify, width};
use serde_json::{Value, json};
use std::io::{Cursor, Write};

fn project(fields: Value) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file(
        "Metadata/project_settings.config",
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    zip.write_all(&serde_json::to_vec(&fields).unwrap())
        .unwrap();
    zip.finish().unwrap().into_inner()
}

fn fields(value: Value) -> Value {
    json!({"line_width": value, "nozzle_diameter": ["0.4"], "layer_height": "0.2",
        "thick_bridges": "0", "thick_internal_bridges": "0", "internal_solid_infill_line_width": "0"})
}

#[test]
fn exported_bytes_require_exact_typed_width_not_physical_equivalence() {
    let dependencies = json!({"nozzle_diameter": ["0.4"], "layer_height": "0.2"});
    for value in [json!("0.6"), json!("150%"), json!("0"), json!("0%")] {
        let result = verify(
            &project(fields(value.clone())),
            "line_width",
            &value,
            dependencies.as_object().unwrap(),
        )
        .unwrap();
        assert_eq!(result["requested"], value);
        assert_eq!(result["effective"], value);
    }
    for (requested, effective) in [
        ("0.6", "150%"),
        ("150%", "0.6"),
        ("150", "150%"),
        ("0", "0%"),
        ("0%", "0"),
    ] {
        assert!(
            verify(
                &project(fields(json!(effective))),
                "line_width",
                &json!(requested),
                dependencies.as_object().unwrap()
            )
            .is_err()
        );
    }
    assert_ne!(width(&json!("0")).unwrap(), width(&json!("0%")).unwrap());
}

#[test]
fn missing_wrong_shape_wrong_owner_default_and_dependency_changes_fail_application() {
    let expected = json!("0.6");
    let dependencies = json!({"nozzle_diameter": ["0.4"], "layer_height": "0.2"});
    for value in [
        json!("0"),
        json!("0.45"),
        json!(["0.6"]),
        json!(0.6),
        Value::Null,
    ] {
        assert!(
            verify(
                &project(fields(value)),
                "line_width",
                &expected,
                dependencies.as_object().unwrap()
            )
            .is_err()
        );
    }
    let mut missing = fields(expected.clone());
    missing.as_object_mut().unwrap().remove("line_width");
    missing["machine_settings"] = json!({"line_width": "0.6"});
    assert!(
        verify(
            &project(missing),
            "line_width",
            &expected,
            dependencies.as_object().unwrap()
        )
        .is_err()
    );
    for (key, value) in [
        ("nozzle_diameter", json!(["0.6"])),
        ("layer_height", json!("0.3")),
    ] {
        let mut changed = fields(expected.clone());
        changed[key] = value;
        assert!(
            verify(
                &project(changed),
                "line_width",
                &expected,
                dependencies.as_object().unwrap()
            )
            .is_err()
        );
    }
    let mut missing_dependency = fields(expected.clone());
    missing_dependency
        .as_object_mut()
        .unwrap()
        .remove("thick_bridges");
    assert!(
        verify(
            &project(missing_dependency),
            "line_width",
            &expected,
            dependencies.as_object().unwrap()
        )
        .is_err()
    );
}
