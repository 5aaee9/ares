//! Printer smoke sweep across OrcaSlicer vendor machine presets.

use std::path::PathBuf;

use crate::{
    self as parity,
    presets::VendorProfiles,
    runner::{self, CaseInputs, OrcaRunner},
};

/// Focused re-check of the nearest-seam / layer-1 deposition cluster
/// after seam-placement changes (env-gated, prints outcomes).
#[test]
fn orca_parity_nearest_cluster_check() {
    if std::env::var("ARES_PARITY_CLUSTER").as_deref() != Ok("1") {
        eprintln!("skipping: set ARES_PARITY_CLUSTER=1");
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap_or_else(|e| panic!("{e}"))
        .expect("explicit cluster requires an Orca executable");
    let targets = [
        ("Anycubic", "Anycubic Kobra 2 Neo 0.4 nozzle"),
        ("Anycubic", "Anycubic Kobra 3 0.4 nozzle"),
    ];
    let root = profiles_root();
    let model = cube_model();
    for (vendor, printer) in targets {
        let Ok(profiles) = VendorProfiles::load(&root, vendor) else {
            eprintln!("SKIP {vendor}/{printer}: load failed");
            continue;
        };
        let outcome = match parity::select_printer(&profiles, vendor, printer) {
            Ok(selection) => {
                if std::env::var("CLUSTER_DUMP").is_ok() {
                    let case = parity::build_selection_case(&runner, &profiles, &selection, &model)
                        .map(|case| {
                            let dir = std::path::Path::new("/tmp/kobra2");
                            std::fs::create_dir_all(dir).unwrap();
                            std::fs::write(dir.join("case.3mf"), &case.project).unwrap();
                            std::fs::write(dir.join("ref.gcode"), &case.reference).unwrap();
                            let runtime = tokio::runtime::Builder::new_current_thread()
                                .build()
                                .expect("tokio runtime");
                            match runtime.block_on(async {
                                ares_core::slice_project(
                                    &case.project,
                                    ares_core::GenerationMetadata::deterministic(
                                        2026, 8, 27, 0, 0, 0,
                                    ),
                                )
                                .await
                            }) {
                                Ok(gcode) => {
                                    let _ = std::fs::write(dir.join("inproc.gcode"), &gcode);
                                    eprintln!("INPROC {} bytes", gcode.len());
                                }
                                Err(error) => eprintln!("INPROC failed: {error}"),
                            }
                        })
                        .map_err(|error| error.to_string());
                    eprintln!("DUMP: {case:?}");
                    assert!(case.is_ok());
                    continue;
                }
                match parity::build_selection_case(&runner, &profiles, &selection, &model) {
                    Ok(case) => parity::compare_case(&case),
                    Err(error) => parity::ares_error(&format!("{vendor}/{printer}"), error),
                }
            }
            Err(error) => parity::ares_error(&format!("{vendor}/{printer}"), error),
        };
        eprintln!(
            "CLUSTER {} {} | {}",
            outcome.status, outcome.label, outcome.detail
        );
    }
}

fn profiles_root() -> PathBuf {
    runner::repo_root().join("OrcaSlicer/resources/profiles")
}

fn cube_model() -> PathBuf {
    runner::repo_root().join("tests/parity/cube10.stl")
}

/// Single-printer smoke case used to develop the harness and record the
/// first divergences on a plain Marlin i3 profile.
#[test]
fn orca_parity_kobra_s1_max_025_smoke() {
    assert_printer_smoke("Anycubic", "Anycubic Kobra S1 Max 0.25 nozzle");
}

#[test]
fn orca_parity_ender3_smoke() {
    assert_printer_smoke("Creality", "Creality Ender-3 0.4 nozzle");
}

#[test]
fn orca_parity_fuzzy_skin_enum_smoke() {
    for value in [
        "none",
        "external",
        "hole",
        "all",
        "allwalls",
        "disabled_fuzzy",
    ] {
        assert_process_option_smoke("fuzzy_skin", value);
    }
}

#[test]
fn orca_parity_internal_octagram_smoke() {
    assert_process_option_smoke("internal_solid_infill_pattern", "octagramspiral");
}

#[test]
fn orca_parity_bottom_hilbert_smoke() {
    assert_process_option_smoke("bottom_surface_pattern", "hilbertcurve");
}

#[test]
fn orca_parity_extra_bridge_layer_smoke() {
    for value in [
        "apply_to_all",
        "disabled",
        "external_bridge_only",
        "internal_bridge_only",
    ] {
        assert_process_option_smoke("enable_extra_bridge_layer", value);
    }
}

#[test]
fn orca_parity_gap_fill_target_smoke() {
    for value in ["everywhere", "nowhere", "topbottom"] {
        assert_process_option_smoke("gap_fill_target", value);
    }
}

#[test]
fn orca_parity_top_concentric_smoke() {
    assert_process_option_smoke("top_surface_pattern", "concentric");
}

#[test]
fn orca_parity_top_rectilinear_smoke() {
    assert_process_option_smoke("top_surface_pattern", "rectilinear");
}

#[test]
fn orca_parity_reduce_crossing_wall_smoke() {
    for value in ["0", "1"] {
        assert_process_option_smoke("reduce_crossing_wall", value);
    }
}

#[test]
fn orca_parity_random_seam_smoke() {
    assert_process_option_smoke("seam_position", "random");
}

#[test]
fn orca_parity_staggered_inner_seams_smoke() {
    for value in ["0", "1"] {
        assert_process_option_smoke("staggered_inner_seams", value);
    }
}

#[test]
fn orca_parity_seam_slope_type_smoke() {
    for value in ["none", "external", "all"] {
        assert_process_option_smoke("seam_slope_type", value);
    }
}

#[test]
fn orca_parity_skirt_loops_smoke() {
    for value in ["0", "4", "10"] {
        assert_process_option_smoke("skirt_loops", value);
    }
}

#[test]
fn orca_parity_zaa_enabled_smoke() {
    for value in ["0", "1"] {
        assert_process_option_smoke("zaa_enabled", value);
    }
}

#[test]
fn orca_parity_spiral_mode_smoke() {
    for value in ["0", "1"] {
        assert_process_option_smoke("spiral_mode", value);
    }
}

#[test]
fn orca_parity_ironing_solid_smoke() {
    assert_process_option_smoke("ironing_type", "solid");
}

#[test]
fn orca_parity_afinia_hs_06_smoke() {
    assert_printer_smoke("Afinia", "Afinia H+1(HS) 0.6 nozzle");
}

#[test]
fn orca_parity_kobra_max_smoke() {
    assert_printer_smoke("Anycubic", "Anycubic Kobra Max 0.4 nozzle");
}

#[test]
fn orca_parity_artillery_x3_pro_smoke() {
    assert_printer_smoke("Artillery", "Artillery Sidewinder X3 Pro 0.4 nozzle");
}

#[test]
fn orca_parity_prusa_core_one_dump() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|e| panic!("{e}")) else {
        eprintln!("skipping: no OrcaSlicer CLI available");
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Prusa").unwrap();
    let selection =
        parity::select_printer(&profiles, "Prusa", "Prusa CORE One 0.4 nozzle").unwrap();
    let result = parity::build_selection_case(&runner, &profiles, &selection, &cube_model())
        .map(|case| {
            let dir = std::path::Path::new("/tmp/prusa");
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(dir.join("case.3mf"), &case.project).unwrap();
            std::fs::write(dir.join("ref.gcode"), &case.reference).unwrap();
        })
        .map_err(|error| error.to_string());
    eprintln!("RESULT: {result:?}");
    assert!(result.is_ok());
}

#[test]
fn orca_parity_ratrig_vcast_smoke() {
    assert_printer_smoke("Ratrig", "RatRig V-Cast 0.4 nozzle");
}

fn assert_printer_smoke(vendor: &str, printer: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|e| panic!("{e}")) else {
        eprintln!("skipping: no OrcaSlicer CLI available");
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), vendor).unwrap();
    let selection = parity::select_printer(&profiles, vendor, printer).unwrap();
    let case = parity::build_selection_case(&runner, &profiles, &selection, &cube_model()).unwrap();
    let outcome = parity::compare_case(&case);
    eprintln!(
        "printer smoke: {} {} {}",
        outcome.status, outcome.label, outcome.detail
    );
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

fn assert_process_option_smoke(key: &str, value: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|e| panic!("{e}")) else {
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert(key.to_owned(), serde_json::Value::String(value.to_owned()));
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let mut overrides = parity::smoke_overrides();
    overrides.remove(key);
    let case = runner
        .build_case(
            &CaseInputs {
                label: &format!("option/{key}/{value}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &overrides,
            &cube_model(),
        )
        .unwrap();
    let outcome = parity::compare_case(&case);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

/// Full vendor sweep; writes `tests/parity/printer-smoke-summary.md` and
/// fails when any printer diverges, so each fix moves the summary to green.
/// Gated behind `ARES_PARITY_SWEEP=1` because slicing every vendor preset
/// through both slicers takes hours.
#[test]
fn orca_parity_printer_sweep() {
    if std::env::var("ARES_PARITY_SWEEP").as_deref() != Ok("1") {
        eprintln!("skipping: set ARES_PARITY_SWEEP=1 to run the full printer sweep");
        return;
    }
    let runner = OrcaRunner::from_env()
        .unwrap_or_else(|e| panic!("{e}"))
        .expect("explicit sweep requires an Orca executable");
    let root = profiles_root();
    let model = cube_model();
    let mut outcomes = Vec::new();
    for vendor in parity::vendors(&root) {
        let Ok(profiles) = VendorProfiles::load(&root, &vendor) else {
            continue;
        };
        for printer in profiles.instantiated_machine_names() {
            let selection = match parity::select_printer(&profiles, &vendor, &printer) {
                Ok(selection) => selection,
                Err(error) => {
                    let outcome = parity::ares_error(&format!("{vendor}/{printer}"), error);
                    eprintln!(
                        "[{}/sweep] {} {}",
                        outcomes.len() + 1,
                        outcome.status,
                        outcome.label
                    );
                    outcomes.push(outcome);
                    continue;
                }
            };
            let outcome = match parity::build_selection_case(&runner, &profiles, &selection, &model)
            {
                Ok(case) => parity::compare_case(&case),
                Err(error) => parity::ares_error(&format!("{vendor}/{printer}"), error),
            };
            eprintln!(
                "[{}/sweep] {} {}",
                outcomes.len() + 1,
                outcome.status,
                outcome.label
            );
            outcomes.push(outcome);
        }
    }
    write_summary(&outcomes);
    let failures = outcomes
        .iter()
        .filter(|outcome| outcome.status != "PASS")
        .count();
    assert!(
        failures == 0,
        "{failures}/{} printers diverge; see tests/parity/printer-smoke-summary.md",
        outcomes.len()
    );
}

fn write_summary(outcomes: &[parity::ParityOutcome]) {
    let passed = outcomes
        .iter()
        .filter(|outcome| outcome.status == "PASS")
        .count();
    let mut summary = format!(
        "# OrcaSlicer printer smoke summary\n\n{} of {} printers pass the semantic parity comparison (classic wall generator baseline; cube model).\n\n> NOTE: timing (M73/model-printing-time) is compared with `compare_ignoring_time` until the GCodeProcessor motion planner reaches Orca parity; timing deltas are therefore not reflected in the divergences below.\n\n| status | printer | first divergence |\n|---|---|---|\n",
        passed,
        outcomes.len()
    );
    for outcome in outcomes {
        summary.push_str(&format!(
            "| {} | {} | {} |\n",
            outcome.status,
            outcome.label,
            outcome.detail.replace('\n', " ")
        ));
    }
    let path = runner::repo_root().join("tests/parity/printer-smoke-summary.md");
    std::fs::write(&path, summary).expect("write printer smoke summary");
}
