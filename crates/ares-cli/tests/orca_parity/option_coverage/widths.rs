//! Bounded PrintConfig/Print::validate width candidates, not Flow success claims.
use serde_json::{Value, json};

use super::{OptionCase, case, seeded_interior};

#[derive(Clone, Debug)]
pub(in crate::option_coverage) struct WidthDomain {
    pub(in crate::option_coverage) metadata: Value,
    pub(in crate::option_coverage) probes: Vec<Value>,
}

// Only the approved single-nozzle context is modeled. Mixed nozzles are deferred.
pub(in crate::option_coverage) fn generate(
    key: &str,
    entry: &Value,
    raw: (f64, f64),
    thick_bridges: bool,
    thick_internal_bridges: bool,
) -> (WidthDomain, Vec<OptionCase>) {
    let bridge = key == "bridge_line_width";
    let lower = if bridge && thick_bridges && thick_internal_bridges {
        0.0
    } else {
        0.2
    };
    let upper = if bridge { 0.4 } else { 2.0 };
    let mut cases = Vec::new();
    let mut units = Vec::new();
    let mut probes = Vec::new();
    for (unit, suffix, scale) in [("mm", "", 1.0), ("percent", "%", 250.0)] {
        let (minimum, maximum) = (lower * scale, upper * scale);
        let seed = seeded_interior(&format!("{key}/{unit}"), minimum, maximum, false);
        for (label, value) in [("zero", 0.0), ("context-max", maximum), ("interior", seed)] {
            let rendered = render(value, suffix);
            cases.push(case(&format!("{unit}-{label}"), &rendered, entry));
        }
        units.push(json!({
            "unit": unit, "raw_schema": {"min": raw.0, "max": raw.1,
                "endpoint_semantics": "ConfigOptionDef::is_value_valid precision=4 approximate endpoints; min=0 rejects negatives"},
            "positive_context": {"lower": minimum, "lower_open": true, "upper": maximum, "upper_open": false},
            "zero": {"wire": render(0.0, suffix), "validation_sentinel": true,
                "flow": if bridge { "solid-infill fallback" } else if suffix.is_empty() { "auto" } else { "zero resolved width; runtime unverified" }}
        }));
        for value in [-1.0, raw.1 + 1.0] {
            probes.push(json!({"wire": render(value, suffix), "expected": "raw-schema-rejection", "execute": false}));
        }
        if raw.1 > maximum {
            probes.push(json!({"wire": render(raw.1, suffix), "expected": "context-rejection", "raw_schema_endpoint": true, "execute": false}));
        }
        if minimum > 0.0 {
            probes.push(json!({"wire": render(minimum, suffix), "expected": "context-rejection", "execute": false}));
        }
    }
    (
        WidthDomain {
            metadata: json!({
                "scope": "single-nozzle validation candidates; not actual-default or slicing coverage",
                "gui_missing_percent_threshold": 10, "units": units,
                "dependencies": {"nozzle_diameter": ["0.4"], "layer_height": "0.2",
                    "thick_bridges": if thick_bridges { "1" } else { "0" },
                    "thick_internal_bridges": if thick_internal_bridges { "1" } else { "0" }},
                "export_required": ["nozzle_diameter", "layer_height", "thick_bridges", "thick_internal_bridges", "internal_solid_infill_line_width"],
                "deferred": ["mixed nozzles", "object/region overrides", "Flow/slicing feasibility", "tolerance edges"]
            }),
            probes,
        },
        cases,
    )
}

fn render(value: f64, suffix: &str) -> String {
    let text = format!("{value:.6}");
    format!(
        "{}{suffix}",
        text.trim_end_matches('0').trim_end_matches('.')
    )
}
