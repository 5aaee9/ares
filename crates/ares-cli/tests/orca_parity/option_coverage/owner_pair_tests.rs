//! Explicit modified-context application proof, never a legal-domain parity PASS.
use super::*;
use crate::runner::{CaseInputs, application};
use sha2::{Digest, Sha256};

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn actual_orca_owner_pair_exports_distinct_typed_widths() {
    if std::env::var("ARES_PARITY_WIDTH_OWNER_PAIR").as_deref() != Ok("1") {
        eprintln!(
            "not executed: set ARES_PARITY_WIDTH_OWNER_PAIR=1 for two fresh actual-Orca cases"
        );
        return;
    }
    let root = parity::artifacts::root_from_env().unwrap();
    let runner = OrcaRunner::from_env()
        .unwrap_or_else(|e| panic!("{e}"))
        .expect("actual Orca and external artifacts required");
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let original_process = profiles.process(&selection.process).unwrap();
    let filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    let original = serde_json::json!({"machine": machine, "process": original_process, "filaments": filaments});
    parity::artifacts::write(
        &root.join("owner-pair-original-presets.json"),
        &serde_json::to_vec_pretty(&original).unwrap(),
    )
    .unwrap();
    let binary = PathBuf::from(
        std::env::var_os("ARES_ORCA_REFERENCE_BINARY").expect("record actual reference binary"),
    );
    let binary_hash = hash(&std::fs::read(&binary).unwrap());
    let mut results = Vec::new();
    for (label, requested) in [("literal", "0.6"), ("percent", "150%")] {
        let mut process = original_process.clone();
        // These are the complete explicit dependent overrides, NOT actual defaults.
        let dependencies = serde_json::json!({"nozzle_diameter": ["0.4"], "layer_height": "0.2",
            "thick_bridges": "0", "thick_internal_bridges": "0"});
        assert_eq!(machine["nozzle_diameter"], dependencies["nozzle_diameter"]);
        for key in ["layer_height", "thick_bridges", "thick_internal_bridges"] {
            process.insert(key.into(), dependencies[key].clone());
        }
        process.insert("line_width".into(), Value::String(requested.into()));
        let exported = runner
            .export_case(
                &CaseInputs {
                    label: &format!("owner-pair/{label}"),
                    machine: &machine,
                    process: &process,
                    filaments: &filaments,
                },
                &Map::new(),
                &runner::repo_root().join("tests/parity/cube10.stl"),
            )
            .unwrap();
        let proof = application::verify(
            &exported.project,
            "line_width",
            &Value::String(requested.into()),
            dependencies.as_object().unwrap(),
        )
        .unwrap();
        parity::artifacts::write(
            &root.join(format!("{label}-application.json")),
            &serde_json::to_vec_pretty(&proof).unwrap(),
        )
        .unwrap();
        let project_hash = hash(&exported.project);
        // Reference slicing and producer comparison are recorded independently from application.
        let result = match runner.slice_case(exported) {
            Ok(case) => {
                let config = String::from_utf8_lossy(&case.reference)
                    .lines()
                    .filter(|line| {
                        [
                            "line_width",
                            "nozzle_diameter",
                            "layer_height",
                            "thick_bridges",
                            "thick_internal_bridges",
                            "internal_solid_infill_line_width",
                        ]
                        .iter()
                        .any(|key| line.starts_with(&format!("; {key} = ")))
                    })
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                let line_width = config
                    .iter()
                    .find_map(|line| line.strip_prefix("; line_width = "))
                    .expect("reference G-code line_width config");
                assert_eq!(
                    application::width(&Value::String(line_width.into())).unwrap(),
                    application::width(&Value::String(requested.into())).unwrap()
                );
                let outcome = parity::compare_case(&case);
                serde_json::json!({"reference_sha256": hash(&case.reference), "gcode_config": config,
                    "producer_status": outcome.status, "producer_detail": outcome.detail,
                    "strict_compared": strict_comparator_ran(outcome.status), "artifacts": outcome.artifacts})
            }
            Err(error) => {
                serde_json::json!({"reference_stage_failure": error.to_string(), "strict_compared": false})
            }
        };
        results.push(
            serde_json::json!({"label": label, "application": proof, "input_sha256": project_hash,
            "result": result, "parity_coverage": "not a completed domain"}),
        );
    }
    parity::artifacts::write(&root.join("owner-pair-results.json"), &serde_json::to_vec_pretty(&serde_json::json!({
        "actual_binary": binary, "actual_binary_sha256": binary_hash, "cases": results,
        "scope": "two fresh process-owner exports; modified single-nozzle context, no default/domain parity claim"
    })).unwrap()).unwrap();
}
