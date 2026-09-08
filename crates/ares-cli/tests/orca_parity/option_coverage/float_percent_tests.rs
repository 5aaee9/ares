use super::{OptionOutcome, domains, strict_comparator_ran};
use crate::runner;

#[test]
fn float_percent_plans_emit_distinct_units_and_contextual_maxima() {
    let plans = domains::load(&runner::repo_root());
    for (key, maximum, percent_maximum) in [
        ("line_width", "2", "500%"),
        ("bridge_line_width", "0.4", "100%"),
    ] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        let values = plan
            .cases
            .iter()
            .filter_map(|case| case.value.as_ref())
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        for required in ["0", "0%", maximum, percent_maximum] {
            assert!(
                values.contains(&required),
                "{key}: missing {required} in {values:?}"
            );
        }
    }
}

#[test]
fn rejected_unexecuted_and_baseline_only_domains_are_incomplete() {
    let plans = domains::load(&runner::repo_root());
    let plan = plans.iter().find(|plan| plan.key == "bridge_flow").unwrap();
    for (compared, rejected) in [(0, vec![]), (0, vec!["rejected".into()]), (1, vec![])] {
        assert_eq!(
            OptionOutcome::executed(plan, compared, rejected, None).status,
            "INCOMPLETE"
        );
    }
    let baseline = plans
        .iter()
        .find(|plan| plan.cases[0].value.is_none())
        .unwrap();
    assert_eq!(
        OptionOutcome::executed(baseline, 1, vec![], None).status,
        "INCOMPLETE"
    );
}

#[test]
fn emitted_schema_context_and_probe_domains_do_not_use_gui_threshold_as_limit() {
    let plans = domains::load(&runner::repo_root());
    for (key, raw_max, contextual_max) in [
        ("line_width", 1000.0, 2.0),
        ("bridge_line_width", 100.0, 0.4),
    ] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        let domain = plan.width_domain.as_ref().unwrap();
        assert_eq!(domain.metadata["gui_missing_percent_threshold"], 10);
        for (index, scale) in [(0, 1.0), (1, 250.0)] {
            let unit = &domain.metadata["units"][index];
            assert_eq!(unit["raw_schema"]["min"], 0.0);
            assert_eq!(unit["raw_schema"]["max"], raw_max);
            assert_eq!(unit["positive_context"]["lower"], 0.2 * scale);
            assert_eq!(unit["positive_context"]["lower_open"], true);
            assert_eq!(unit["positive_context"]["upper"], contextual_max * scale);
            assert_eq!(unit["positive_context"]["upper_open"], false);
        }
        assert!(domain.probes.iter().all(|probe| probe["execute"] == false));
        assert!(
            domain
                .probes
                .iter()
                .any(|probe| probe["expected"] == "raw-schema-rejection")
        );
        assert!(
            domain
                .probes
                .iter()
                .any(|probe| probe["expected"] == "context-rejection"
                    && probe["raw_schema_endpoint"] == true)
        );
        assert_eq!(plan.raw_scope, "process");
        for case in plan
            .cases
            .iter()
            .filter(|case| case.label.ends_with("interior"))
        {
            let value = runner::application::width(case.value.as_ref().unwrap()).unwrap();
            let scale = if value.percent { 250.0 } else { 1.0 };
            assert!(value.value > 0.2 * scale && value.value < contextual_max * scale);
        }
        eprintln!(
            "{}",
            serde_json::json!({"key": key, "source": plan.source, "domain": domain.metadata,
            "probes": domain.probes, "cases": plan.cases.iter().map(|case|
                serde_json::json!({"label": case.label, "wire": case.value})).collect::<Vec<_>>() })
        );
    }
}

#[test]
fn bridge_lower_boundary_requires_both_thick_flags_and_interiors_preserve_units() {
    let entry = serde_json::json!({"wire_shape": "scalar"});
    for (thick, internal) in [(false, false), (false, true), (true, false), (true, true)] {
        let (domain, cases) =
            domains::widths::generate("bridge_line_width", &entry, (0.0, 100.0), thick, internal);
        let lower = if thick && internal { 0.0 } else { 0.2 };
        assert_eq!(
            domain.metadata["units"][0]["positive_context"]["lower"],
            lower
        );
        assert_eq!(
            domain.metadata["units"][0]["positive_context"]["upper"],
            0.4
        );
        let (_, repeated) =
            domains::widths::generate("bridge_line_width", &entry, (0.0, 100.0), thick, internal);
        for (case, repeated) in cases.iter().zip(repeated) {
            assert_eq!(case.value, repeated.value);
            if !case.label.ends_with("interior") {
                continue;
            }
            let parsed = runner::application::width(case.value.as_ref().unwrap()).unwrap();
            let scale = if parsed.percent { 250.0 } else { 1.0 };
            assert!(parsed.value > lower * scale && parsed.value < 0.4 * scale);
            assert_eq!(parsed.percent, case.label.starts_with("percent-"));
        }
    }
}

#[test]
fn producer_error_does_not_count_as_comparison_or_application_failure() {
    assert!(strict_comparator_ran("PASS"));
    assert!(strict_comparator_ran("DIVERGENT"));
    for status in [
        "ARES_ERROR",
        "ARTIFACT_ERROR",
        "INPUT_REJECTED",
        "APPLICATION_MISMATCH",
    ] {
        assert!(!strict_comparator_ran(status));
    }
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "line_width")
        .unwrap();
    let error = OptionOutcome::executed(&plan, 0, vec![], Some("applied + producer error".into()));
    assert_eq!(error.status, "FAIL");
    assert_eq!(error.compared, 0);
    assert_eq!(error.rejected, 0);
    assert_eq!(
        OptionOutcome::executed(&plan, plan.cases.len(), vec![], None).status,
        "PASS"
    );
}
