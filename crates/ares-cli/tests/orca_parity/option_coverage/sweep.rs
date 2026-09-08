use super::*;
use crate::runner::CaseInputs;

#[test]
fn orca_parity_option_coverage() {
    if std::env::var("ARES_PARITY_OPTIONS").as_deref() != Ok("1") {
        eprintln!("skipping: set ARES_PARITY_OPTIONS=1 to run option coverage");
        return;
    }
    let runner = OrcaRunner::from_env()
        .expect("explicit option coverage requires Orca and external artifacts");
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let plans = domains::load(&runner::repo_root());
    let outcomes = plans
        .iter()
        .map(|plan| execute_plan(&runner, plan, &machine, &process, &filaments))
        .collect::<Vec<_>>();
    write_summary(&outcomes);
    let incomplete = outcomes
        .iter()
        .filter(|outcome| outcome.status != "PASS")
        .count();
    assert_eq!(
        incomplete, 0,
        "{incomplete} option domains incomplete or failed; see external option-coverage-summary.md"
    );
}

fn execute_plan(
    runner: &OrcaRunner,
    plan: &domains::OptionPlan,
    machine: &Map<String, Value>,
    process: &Map<String, Value>,
    filaments: &[Map<String, Value>],
) -> OptionOutcome {
    if plan.cases.is_empty() {
        return OptionOutcome::omitted(plan);
    }
    let mut first_failure = None;
    let mut compared = 0;
    let mut rejected = Vec::new();
    let io_error = |error| {
        runner::stages::StageError::new(
            runner::stages::Stage::Application,
            runner::stages::FailureKind::Io,
            error,
        )
    };
    for case in &plan.cases {
        let mut case_machine = machine.clone();
        let mut case_process = process.clone();
        let mut case_filaments = filaments.to_vec();
        inject_case(
            plan,
            case,
            &mut case_machine,
            &mut case_process,
            &mut case_filaments,
        );
        let label = format!("option/{}/{}", plan.key, case.label);
        let mut overrides = parity::smoke_overrides();
        overrides.remove(&plan.key);
        let built = runner
            .export_case(
                &CaseInputs {
                    label: &label,
                    machine: &case_machine,
                    process: &case_process,
                    filaments: &case_filaments,
                },
                &overrides,
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .and_then(|exported| {
                if let Some(domain) = &plan.width_domain {
                    let proof = runner::application::verify(
                        &exported.project,
                        &plan.key,
                        case.value.as_ref().unwrap(),
                        domain.metadata["dependencies"].as_object().unwrap(),
                    )?;
                    let root = parity::artifacts::root_from_env().map_err(io_error)?;
                    let file = root.join(format!("{}-{}-application.json", plan.key, case.label));
                    parity::artifacts::write(&file, &serde_json::to_vec_pretty(&proof).unwrap())
                        .map_err(io_error)?;
                }
                runner.slice_case(exported)
            });
        match built {
            Ok(built) => {
                let outcome = parity::compare_case(&built);
                compared += usize::from(strict_comparator_ran(outcome.status));
                eprintln!("[option] {} {label}", outcome.status);
                if outcome.status != "PASS" && first_failure.is_none() {
                    first_failure = Some(format!("{}: {}", case.label, outcome.detail));
                }
            }
            Err(error) if error.input_rejection(&plan.key) => {
                rejected.push(format!("{}: {error}", case.label));
                eprintln!("[option] INPUT_REJECTED {label}");
            }
            Err(error) => {
                eprintln!("[option] STAGE_FAILURE {label}: {error}");
                first_failure.get_or_insert_with(|| format!("{}: {error}", case.label));
            }
        }
    }
    OptionOutcome::executed(plan, compared, rejected, first_failure)
}
