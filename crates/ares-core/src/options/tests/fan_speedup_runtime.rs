use super::super::*;
use serde_json::{Value, json};

#[test]
fn fan_speedup_control_defaults_to_overhang_only_disabled_time() {
    let control = SliceOptions::default().fan_speedup_control().unwrap();

    assert_eq!(control.time_s(), 0.0);
    assert!(control.only_overhangs());
}

#[test]
fn fan_speedup_control_accepts_non_negative_scalar_seconds_and_bool_gate() {
    let options: SliceOptions = serde_json::from_value(json!({
        "fan_speedup_time": "0.25",
        "fan_speedup_overhangs": false
    }))
    .unwrap();
    let control = options.fan_speedup_control().unwrap();

    assert_eq!(control.time_s(), 0.25);
    assert!(!control.only_overhangs());
}

#[test]
fn fan_speedup_control_rejects_invalid_values() {
    for (key, value) in [
        ("fan_speedup_time", json!(-0.01)),
        ("fan_speedup_time", json!("nan")),
        ("fan_speedup_time", json!([0.1])),
        ("fan_speedup_time", Value::Null),
        ("fan_speedup_overhangs", json!("true")),
        ("fan_speedup_overhangs", json!(1)),
        ("fan_speedup_overhangs", Value::Null),
    ] {
        let mut options = serde_json::Map::new();
        options.insert(key.to_owned(), value);
        let options: SliceOptions = serde_json::from_value(Value::Object(options)).unwrap();

        let err = options.fan_speedup_control().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}

#[test]
fn project_settings_route_fan_speedup_time_into_printer_gcode() {
    let settings: crate::ProjectSettings = serde_json::from_value(json!({
        "fan_speedup_time": "0.3", "fan_kickstart": "0.5"
    }))
    .unwrap();
    assert_eq!(settings.printer.gcode.fan_speedup_time.0, 0.3);
    assert_eq!(settings.printer.gcode.fan_kickstart.0, 0.5);
}
