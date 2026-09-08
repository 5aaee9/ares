//! Option-domain parity sweep against the Ender-3 smoke fixture.

#[path = "option_coverage/domains.rs"]
mod domains;
#[cfg(test)]
#[path = "option_coverage/float_percent_tests.rs"]
mod float_percent_tests;
#[cfg(test)]
#[path = "option_coverage/owner_pair_tests.rs"]
mod owner_pair_tests;
#[cfg(test)]
#[path = "option_coverage/sweep.rs"]
mod sweep;
#[cfg(test)]
#[path = "option_coverage/tests.rs"]
mod tests;

use crate::{
    self as parity,
    presets::VendorProfiles,
    runner::{self, OrcaRunner},
};
use serde_json::{Map, Value};
use std::path::PathBuf;

fn profiles_root() -> PathBuf {
    runner::repo_root().join("OrcaSlicer/resources/profiles")
}

fn inject_case(
    plan: &domains::OptionPlan,
    case: &domains::OptionCase,
    machine: &mut Map<String, Value>,
    process: &mut Map<String, Value>,
    filaments: &mut [Map<String, Value>],
) {
    let target = match plan.raw_scope.as_str() {
        "printer" => machine,
        "process" => process,
        "filament" => &mut filaments[0],
        "residual" if residual_is_machine(&plan.key) => machine,
        "residual" => process,
        scope => panic!("unknown option scope {scope}"),
    };
    if let Some(value) = &case.value {
        target.insert(plan.key.clone(), value.clone());
    }
}

fn residual_is_machine(key: &str) -> bool {
    matches!(
        key,
        "deretraction_speed"
            | "extruder_offset"
            | "max_layer_height"
            | "min_layer_height"
            | "nozzle_diameter"
            | "nozzle_volume_type"
            | "retract_before_wipe"
            | "retract_length_toolchange"
            | "retract_lift_above"
            | "retract_lift_below"
            | "retract_restart_extra"
            | "retract_restart_extra_toolchange"
            | "retraction_length"
            | "retraction_minimum_travel"
            | "retraction_speed"
            | "wipe"
            | "wipe_distance"
            | "z_hop"
    )
}

fn strict_comparator_ran(status: &str) -> bool {
    matches!(status, "PASS" | "DIVERGENT")
}

struct OptionOutcome {
    key: String,
    option_type: String,
    source: String,
    cases: usize,
    compared: usize,
    rejected: usize,
    status: &'static str,
    detail: String,
}

impl OptionOutcome {
    fn executed(
        plan: &domains::OptionPlan,
        compared: usize,
        rejected: Vec<String>,
        failure: Option<String>,
    ) -> Self {
        let (status, detail) = if let Some(detail) = failure {
            ("FAIL", detail)
        } else if compared == 0
            || compared != plan.cases.len()
            || !rejected.is_empty()
            || plan.cases.iter().any(|case| case.value.is_none())
        {
            (
                "INCOMPLETE",
                format!(
                    "required legal cases not all compared; {}",
                    rejected.join("; ")
                ),
            )
        } else if plan.width_domain.is_none() {
            // Legacy generation has no effective-application proof and does not
            // establish full legality for its numeric or unbounded domains.
            (
                "INCOMPLETE",
                "legacy domain application/legality unverified".into(),
            )
        } else {
            ("PASS", String::new())
        };
        Self::new(
            plan,
            (plan.cases.len(), compared, rejected.len()),
            status,
            detail,
        )
    }

    fn omitted(plan: &domains::OptionPlan) -> Self {
        Self::new(
            plan,
            (0, 0, 0),
            "UNBOUNDED",
            plan.omission.unwrap_or_default().to_owned(),
        )
    }

    fn new(
        plan: &domains::OptionPlan,
        counts: (usize, usize, usize),
        status: &'static str,
        detail: String,
    ) -> Self {
        Self {
            key: plan.key.clone(),
            option_type: plan.option_type.clone(),
            source: plan.source.clone(),
            cases: counts.0,
            compared: counts.1,
            rejected: counts.2,
            status,
            detail,
        }
    }
}

fn write_summary(outcomes: &[OptionOutcome]) {
    let pass = outcomes
        .iter()
        .filter(|outcome| outcome.status == "PASS")
        .count();
    let generated = outcomes.iter().map(|outcome| outcome.cases).sum::<usize>();
    let compared = outcomes
        .iter()
        .map(|outcome| outcome.compared)
        .sum::<usize>();
    let rejected = outcomes
        .iter()
        .map(|outcome| outcome.rejected)
        .sum::<usize>();
    let mut output = format!(
        "# OrcaSlicer option coverage summary\n\n{pass}/{} bounded domains complete; {generated} generated, {compared} strictly compared, {rejected} rejected. Not all-printer/default coverage.\n\n| status | option | type | cases | compared | rejected | upstream | first result |\n|---|---|---|---:|---:|---:|---|---|\n",
        outcomes.len()
    );
    for outcome in outcomes {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            outcome.status,
            outcome.key,
            outcome.option_type,
            outcome.cases,
            outcome.compared,
            outcome.rejected,
            outcome.source,
            outcome.detail.replace(['\n', '|'], " ")
        ));
    }
    let root = parity::artifacts::root_from_env().unwrap();
    parity::artifacts::write(&root.join("option-coverage-summary.md"), output.as_bytes()).unwrap();
    let plans = domains::load(&runner::repo_root());
    let widths = plans.iter().filter_map(|plan| plan.width_domain.as_ref().map(|domain| {
        serde_json::json!({"key": plan.key, "domain": domain.metadata, "nonexecuted_probes": domain.probes})
    })).collect::<Vec<_>>();
    parity::artifacts::write(
        &root.join("width-domains.json"),
        &serde_json::to_vec_pretty(&widths).unwrap(),
    )
    .unwrap();
}
