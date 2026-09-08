use std::path::Path;

use regex::Regex;
use serde_json::Value;

#[path = "widths.rs"]
pub(super) mod widths;

#[derive(Clone, Debug)]
pub(super) struct OptionPlan {
    pub(super) key: String,
    pub(super) option_type: String,
    pub(super) raw_scope: String,
    pub(super) source: String,
    pub(super) cases: Vec<OptionCase>,
    pub(super) omission: Option<&'static str>,
    pub(super) width_domain: Option<widths::WidthDomain>,
}

#[derive(Clone, Debug)]
pub(super) struct OptionCase {
    pub(super) label: String,
    pub(super) value: Option<Value>,
}

pub(super) fn load(repo: &Path) -> Vec<OptionPlan> {
    let inventory: Value = serde_json::from_str(
        &std::fs::read_to_string(repo.join("tests/ksr_fdmtest_v4/options-v242.json")).unwrap(),
    )
    .unwrap();
    let source =
        std::fs::read_to_string(repo.join("OrcaSlicer/src/libslic3r/PrintConfig.cpp")).unwrap();
    let lines = source.lines().collect::<Vec<_>>();
    inventory
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| plan(entry, &lines, &source))
        .collect()
}

fn plan(entry: &Value, lines: &[&str], source_text: &str) -> Option<OptionPlan> {
    let key = entry.get("key")?.as_str()?.to_owned();
    let option_type = entry.get("option_type")?.as_str()?.to_owned();
    let raw_scope = entry.get("raw_scope")?.as_str()?.to_owned();
    if option_type == "Metadata" {
        return None;
    }
    let definition = entry.get("upstream_definition")?;
    let line = definition.get("line")?.as_u64()? as usize;
    let source = format!("{}:{}", definition.get("path")?.as_str()?, line);
    let block = definition_block(lines, &key, line);
    let mut width_domain = None;
    let (values, omission) = if matches!(key.as_str(), "line_width" | "bridge_line_width") {
        let raw = raw_numeric_bounds(&block).expect("width schema must have explicit raw bounds");
        let (domain, values) = widths::generate(&key, entry, raw, false, false);
        width_domain = Some(domain);
        (values, None)
    } else if matches!(option_type.as_str(), "coBool" | "coBools") {
        (
            vec![case("false", "0", entry), case("true", "1", entry)],
            None,
        )
    } else if matches!(option_type.as_str(), "coEnum" | "coEnums") {
        let values = enum_values(&key, &block, source_text)
            .into_iter()
            .map(|value| case(&value, &value, entry))
            .collect::<Vec<_>>();
        let omission = values
            .is_empty()
            .then_some("enum domain not explicit in definition block");
        (values, omission)
    } else if range_type(&option_type) {
        match numeric_bounds(&block) {
            Some((minimum, maximum)) if minimum < maximum => {
                let interior = seeded_interior(&key, minimum, maximum, option_type.contains("Int"));
                (
                    vec![
                        numeric_case("min", minimum, entry, &option_type),
                        numeric_case("max", maximum, entry, &option_type),
                        numeric_case("seeded", interior, entry, &option_type),
                    ],
                    None,
                )
            }
            _ => (vec![baseline_case()], None),
        }
    } else {
        (vec![baseline_case()], None)
    };
    Some(OptionPlan {
        key,
        option_type,
        raw_scope,
        source,
        cases: values,
        omission,
        width_domain,
    })
}

fn definition_block(lines: &[&str], key: &str, reported_line: usize) -> String {
    let needle = format!("add(\"{key}\"");
    let lower = reported_line.saturating_sub(20);
    let upper = (reported_line + 20).min(lines.len());
    let start = (lower..upper)
        .find(|index| lines[*index].contains(&needle))
        .unwrap_or_else(|| reported_line.saturating_sub(1));
    let end = (start + 1..lines.len())
        .find(|index| lines[*index].contains("this->add(\"") || lines[*index].contains("add(\""))
        .unwrap_or(lines.len());
    lines[start..end.min(start + 160)].join("\n")
}

fn enum_values(key: &str, block: &str, source: &str) -> Vec<String> {
    let push = Regex::new(r#"enum_values\.(?:push_back|emplace_back)\("([^"]+)"\)"#).unwrap();
    let quoted = Regex::new(r#""([^"]+)""#).unwrap();
    let active_block = strip_comments(block);
    let mut values = push
        .captures_iter(&active_block)
        .map(|captures| captures[1].to_owned())
        .collect::<Vec<_>>();
    if values.is_empty() {
        let assignment = Regex::new(r"enum_values\s*=\s*\{").unwrap();
        if let Some(found) = assignment.find(&active_block)
            && let Some(end) = active_block[found.end()..].find("};")
        {
            values.extend(
                quoted
                    .captures_iter(&active_block[found.end()..found.end() + end])
                    .map(|captures| captures[1].to_owned()),
            );
        }
    }
    if values.is_empty() {
        values.extend(referenced_enum_values(&active_block, source, &push));
    }
    if values.is_empty()
        && let Some(kind) = Regex::new(r"ConfigOptionEnum<([^>]+)>")
            .unwrap()
            .captures(&active_block)
    {
        let kind = kind[1].rsplit("::").next().unwrap();
        let map = Regex::new(&format!(
            r"(?s)s_keys_map_{}\s*\{{(.*?)\}};",
            regex::escape(kind)
        ))
        .unwrap();
        if let Some(body) = map.captures(source) {
            values.extend(
                quoted
                    .captures_iter(&body[1])
                    .map(|captures| captures[1].to_owned()),
            );
        }
    }
    if values.is_empty() {
        values.extend(
            known_vector_enum(key)
                .iter()
                .map(|value| (*value).to_owned()),
        );
    }
    values.sort();
    values.dedup();
    values
}

fn strip_comments(source: &str) -> String {
    Regex::new(r"(?s)/\*.*?\*/|//[^\n]*")
        .unwrap()
        .replace_all(source, "")
        .into_owned()
}

fn referenced_enum_values(block: &str, source: &str, push: &Regex) -> Vec<String> {
    let Some(reference) = Regex::new(r"enum_values\s*=\s*(\w+)->enum_values")
        .unwrap()
        .captures(block)
        .map(|captures| captures[1].to_owned())
    else {
        return Vec::new();
    };
    let marker = Regex::new(&format!(
        r"(?:auto\s+)?{}\s*=\s*def\s*=\s*this->add\(",
        regex::escape(&reference)
    ))
    .unwrap();
    let Some(start) = marker.find(source).map(|found| found.start()) else {
        return Vec::new();
    };
    let tail = &source[start..];
    let end = tail[marker.find(tail).unwrap().end()..]
        .find("this->add(\"")
        .map_or(tail.len(), |offset| {
            marker.find(tail).unwrap().end() + offset
        });
    let active = strip_comments(&tail[..end]);
    push.captures_iter(&active)
        .map(|captures| captures[1].to_owned())
        .collect()
}

fn known_vector_enum(key: &str) -> &'static [&'static str] {
    match key {
        "default_nozzle_volume_type" | "nozzle_volume_type" => &["Standard", "High Flow"],
        "filament_retract_lift_enforce" => {
            &["All Surfaces", "Top Only", "Bottom Only", "Top and Bottom"]
        }
        "filament_z_hop_types" => &["Auto Lift", "Normal Lift", "Slope Lift", "Spiral Lift"],
        _ => &[],
    }
}

fn raw_numeric_bounds(block: &str) -> Option<(f64, f64)> {
    let bound = |name: &str| {
        Regex::new(&format!(r"def->{name}\s*=\s*([-+]?\d+(?:\.\d+)?)"))
            .unwrap()
            .captures(block)
            .and_then(|captures| captures[1].parse::<f64>().ok())
    };
    Some((bound("min")?, bound("max")?))
}

// Legacy domains outside the two-key vertical remain unverified.
fn numeric_bounds(block: &str) -> Option<(f64, f64)> {
    let bound = |name: &str| {
        Regex::new(&format!(r"def->{name}\s*=\s*([-+]?\d+(?:\.\d+)?)"))
            .unwrap()
            .captures(block)
            .and_then(|captures| captures[1].parse::<f64>().ok())
    };
    let minimum = bound("min")?;
    let mut maximum = bound("max")?;
    if let Some(literal) = bound("max_literal") {
        maximum = maximum.min(literal);
    }
    Some((minimum, maximum))
}

fn seeded_interior(key: &str, minimum: f64, maximum: f64, integer: bool) -> f64 {
    let hash = key.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    });
    let ratio = 0.2 + (hash % 600_001) as f64 / 1_000_000.0;
    let value = minimum + ratio * (maximum - minimum);
    if integer {
        value.round().clamp(minimum + 1.0, maximum - 1.0)
    } else {
        value
    }
}

fn numeric_case(label: &str, value: f64, entry: &Value, option_type: &str) -> OptionCase {
    let rendered = if option_type.contains("Int") {
        format!("{value:.0}")
    } else if option_type.contains("Percent") {
        format!("{value}%")
    } else {
        value.to_string()
    };
    case(label, &rendered, entry)
}

fn case(label: &str, value: &str, entry: &Value) -> OptionCase {
    let scalar = Value::String(value.to_owned());
    let value = if entry.get("wire_shape").and_then(Value::as_str) == Some("array") {
        Value::Array(vec![scalar])
    } else {
        scalar
    };
    OptionCase {
        label: label.to_owned(),
        value: Some(value),
    }
}

fn baseline_case() -> OptionCase {
    OptionCase {
        label: "baseline".to_owned(),
        value: None,
    }
}

fn range_type(option_type: &str) -> bool {
    matches!(
        option_type,
        "coFloat"
            | "coFloats"
            | "coInt"
            | "coInts"
            | "coPercent"
            | "coPercents"
            | "coFloatOrPercent"
    )
}
