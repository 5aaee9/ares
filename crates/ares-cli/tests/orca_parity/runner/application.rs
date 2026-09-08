//! ConfigOptionFloatOrPercent tag/magnitude checks at bbs_3mf export bytes.
use super::stages::{FailureKind, Stage, StageError};
use serde_json::{Map, Value};
use std::io::{Cursor, Read};

#[derive(Debug, PartialEq)]
pub(crate) struct WidthValue {
    pub(crate) value: f64,
    pub(crate) percent: bool,
}

pub(crate) fn width(value: &Value) -> Result<WidthValue, String> {
    let text = value.as_str().ok_or("expected scalar string")?;
    let (number, percent) = text
        .strip_suffix('%')
        .map_or((text, false), |number| (number, true));
    let value = number.parse::<f64>().map_err(|error| error.to_string())?;
    if !value.is_finite() {
        return Err("nonfinite width".into());
    }
    Ok(WidthValue { value, percent })
}

pub(crate) fn settings(project: &[u8]) -> Result<Map<String, Value>, StageError> {
    let error = |message| {
        StageError::new(
            Stage::Application,
            FailureKind::ApplicationMismatch,
            message,
        )
    };
    let mut archive =
        zip::ZipArchive::new(Cursor::new(project)).map_err(|e| error(e.to_string()))?;
    let mut bytes = Vec::new();
    archive
        .by_name("Metadata/project_settings.config")
        .map_err(|e| error(e.to_string()))?
        .read_to_end(&mut bytes)
        .map_err(|e| error(e.to_string()))?;
    serde_json::from_slice(&bytes).map_err(|e| error(e.to_string()))
}

pub(crate) fn verify(
    project: &[u8],
    key: &str,
    requested: &Value,
    dependencies: &Map<String, Value>,
) -> Result<Value, StageError> {
    let effective = settings(project)?;
    let mismatch =
        |detail| StageError::new(Stage::Application, FailureKind::ApplicationMismatch, detail);
    let expected = width(requested).map_err(mismatch)?;
    let actual_value = effective
        .get(key)
        .ok_or_else(|| mismatch(format!("missing {key}")))?;
    let actual = width(actual_value).map_err(mismatch)?;
    if expected != actual {
        return Err(mismatch(format!(
            "{key}: requested {requested}, effective {actual_value}"
        )));
    }
    for (key, expected) in dependencies {
        if effective.get(key) != Some(expected) {
            return Err(mismatch(format!(
                "dependency {key}: expected {expected}, effective {:?}",
                effective.get(key)
            )));
        }
    }
    let mut exported = Map::new();
    for key in [
        "nozzle_diameter",
        "layer_height",
        "thick_bridges",
        "thick_internal_bridges",
        "internal_solid_infill_line_width",
    ] {
        let value = effective
            .get(key)
            .ok_or_else(|| mismatch(format!("missing dependency {key}")))?;
        exported.insert(key.into(), value.clone());
    }
    Ok(
        serde_json::json!({"key": key, "requested": requested, "effective": actual_value,
        "typed": {"value": actual.value, "percent": actual.percent}, "dependencies": exported,
        "scope": "project export; object/region activation not proven"}),
    )
}
