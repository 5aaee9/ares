//! Behavioral regressions invoke the actual replay entrypoint in this build.
use std::{fs, path::Path, process::Command};

use serde_json::Value;
use sha2::{Digest, Sha256};

fn replay(fixtures: &Path, artifacts: &Path, cwd: &Path) -> std::process::Output {
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "replay::orca_parity_replay_sweep", "--nocapture"])
        .env("ARES_PARITY_REPLAY", fixtures)
        .env("ARES_PARITY_ARTIFACT_ROOT", artifacts)
        .current_dir(cwd)
        .output()
        .unwrap()
}

fn fixture(root: &Path, reference: bool) {
    fs::create_dir_all(root.join("ender3")).unwrap();
    fs::copy(
        crate::runner::repo_root().join("tests/parity/replay/ender3-classic.3mf"),
        root.join("ender3.3mf"),
    )
    .unwrap();
    if reference {
        fs::copy(
            crate::runner::repo_root().join("tests/parity/replay/ender3-classic.orca.gcode"),
            root.join("ender3/plate_1.gcode"),
        )
        .unwrap();
    }
}

fn assert_failed(output: std::process::Output) {
    assert!(
        !output.status.success(),
        "explicit replay falsely succeeded:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn explicit_empty_inventory_fails() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fs::create_dir(&input).unwrap();
    assert_failed(replay(&input, &temp.path().join("artifacts"), temp.path()));
}

#[test]
fn explicit_missing_reference_fails() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fixture(&input, false);
    let artifacts = temp.path().join("artifacts");
    assert_failed(replay(&input, &artifacts, temp.path()));
    let (case, manifest) = saved_case(&artifacts);
    assert_eq!(manifest["status"], "ARES_ERROR");
    assert!(
        manifest["detail"]
            .as_str()
            .unwrap()
            .contains("reference gcode")
    );
    assert!(case.join("input.3mf").is_file());
    assert!(!case.join("ares.gcode").exists());
    assert!(case.join("error.txt").is_file());
}

#[test]
fn explicit_unwritable_artifact_root_fails() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fixture(&input, true);
    let artifacts = temp.path().join("artifacts");
    fs::write(&artifacts, "not a directory").unwrap();
    assert_failed(replay(&input, &artifacts, temp.path()));
}

#[test]
fn explicit_divergent_orca_output_fails() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fixture(&input, true);
    let reference = input.join("ender3/plate_1.gcode");
    let original = fs::read_to_string(&reference).unwrap();
    // One observable mutation to a real independently generated Orca artifact.
    let changed = original.replacen("M104 S", "M104 S9", 1);
    assert_ne!(original, changed);
    fs::write(reference, &changed).unwrap();
    let artifacts = temp.path().join("artifacts");
    assert_failed(replay(&input, &artifacts, temp.path()));
    let (case, manifest) = saved_case(&artifacts);
    assert_eq!(manifest["status"], "DIVERGENT");
    assert_eq!(
        fs::read(case.join("orca.gcode")).unwrap(),
        changed.as_bytes()
    );
    assert!(case.join("ares.gcode").is_file());
    assert!(case.join("error.txt").is_file());
}

#[test]
fn successful_partial_check_saves_complete_paired_bytes_and_hashes() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fixture(&input, true);
    let artifacts = temp.path().join("artifacts");
    let output = replay(&input, &artifacts, temp.path());
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let summary: Value =
        serde_json::from_slice(&fs::read(artifacts.join("replay-summary.json")).unwrap()).unwrap();
    assert_eq!(summary["evidence"], "partial_semantic_only");
    assert_eq!(summary["compared"], 1);
    let case = Path::new(summary["cases"][0]["artifacts"].as_str().unwrap());
    let manifest: Value =
        serde_json::from_slice(&fs::read(case.join("manifest.json")).unwrap()).unwrap();
    for name in ["input.3mf", "orca.gcode", "ares.gcode"] {
        let bytes = fs::read(case.join(name)).unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(manifest["files"][name]["bytes"], bytes.len());
        assert_eq!(
            manifest["files"][name]["sha256"],
            Sha256::digest(&bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
    }
    assert_eq!(
        fs::read(case.join("input.3mf")).unwrap(),
        fs::read(input.join("ender3.3mf")).unwrap()
    );
    assert_eq!(
        fs::read(case.join("orca.gcode")).unwrap(),
        fs::read(input.join("ender3/plate_1.gcode")).unwrap()
    );
    let project = fs::read(input.join("ender3.3mf")).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let actual = runtime
        .block_on(ares_core::slice_project(
            &project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        ))
        .unwrap();
    assert_eq!(fs::read(case.join("ares.gcode")).unwrap(), actual);
    assert_eq!(manifest["reference_provenance"]["status"], "unknown");
    assert_eq!(manifest["evidence"], "partial_semantic_only");
    assert!(
        manifest["ares_build"]["executable_sha256"]
            .as_str()
            .unwrap()
            .len()
            == 64
    );
}

fn saved_case(root: &Path) -> (std::path::PathBuf, Value) {
    let summary: Value =
        serde_json::from_slice(&fs::read(root.join("replay-summary.json")).unwrap()).unwrap();
    let case = std::path::PathBuf::from(summary["cases"][0]["artifacts"].as_str().unwrap());
    let manifest = serde_json::from_slice(&fs::read(case.join("manifest.json")).unwrap()).unwrap();
    (case, manifest)
}

#[test]
fn original_orca_project_rejection_is_failure_with_error_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fixture(&input, true);
    fs::copy(
        crate::runner::repo_root().join("tests/parity/orca_cli_ender3.3mf"),
        input.join("ender3.3mf"),
    )
    .unwrap();
    fs::copy(
        crate::runner::repo_root().join("tests/parity/replay/ender3.orca.gcode"),
        input.join("ender3/plate_1.gcode"),
    )
    .unwrap();
    let artifacts = temp.path().join("artifacts");
    assert_failed(replay(&input, &artifacts, temp.path()));
    let (case, manifest) = saved_case(&artifacts);
    assert_eq!(manifest["status"], "ARES_ERROR");
    assert!(
        manifest["detail"]
            .as_str()
            .unwrap()
            .contains("wall_generator")
    );
    assert!(case.join("input.3mf").is_file());
    assert!(case.join("orca.gcode").is_file());
    assert!(!case.join("ares.gcode").exists());
    assert!(
        fs::read_to_string(case.join("error.txt"))
            .unwrap()
            .contains("wall_generator")
    );
}

#[test]
fn explicit_unreadable_project_fails_and_preserves_reference() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fixture(&input, true);
    fs::remove_file(input.join("ender3.3mf")).unwrap();
    fs::create_dir(input.join("ender3.3mf")).unwrap();
    let artifacts = temp.path().join("artifacts");
    assert_failed(replay(&input, &artifacts, temp.path()));
    let (case, manifest) = saved_case(&artifacts);
    assert_eq!(manifest["status"], "ARES_ERROR");
    assert!(manifest["detail"].as_str().unwrap().contains("fixture 3mf"));
    assert!(case.join("orca.gcode").is_file());
    assert!(!case.join("ares.gcode").exists());
}

#[test]
fn explicit_unwritable_report_fails_after_saving_paired_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fixture(&input, true);
    let artifacts = temp.path().join("artifacts");
    fs::create_dir_all(artifacts.join("replay-summary.json")).unwrap();
    let output = replay(&input, &artifacts, temp.path());
    assert!(String::from_utf8_lossy(&output.stderr).contains("replay-summary.json"));
    assert_failed(output);
    let case = fs::read_dir(&artifacts)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|path| path.join("manifest.json").exists())
        .unwrap();
    for name in ["input.3mf", "orca.gcode", "ares.gcode"] {
        assert!(case.join(name).is_file());
    }
}

#[test]
fn explicit_missing_inventory_fails() {
    let temp = tempfile::tempdir().unwrap();
    let output = replay(
        &temp.path().join("missing"),
        &temp.path().join("artifacts"),
        temp.path(),
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing"));
    assert_failed(output);
}

#[test]
fn explicit_request_without_artifact_destination_fails() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fixture(&input, true);
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "replay::orca_parity_replay_sweep", "--nocapture"])
        .env("ARES_PARITY_REPLAY", &input)
        .env_remove("ARES_PARITY_ARTIFACT_ROOT")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stderr).contains("ARES_PARITY_ARTIFACT_ROOT"));
    assert_failed(output);
}

#[test]
fn unset_replay_is_explicitly_offline_not_a_comparison() {
    let temp = tempfile::tempdir().unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "replay::orca_parity_replay_sweep", "--nocapture"])
        .env_remove("ARES_PARITY_REPLAY")
        .env_remove("ARES_PARITY_ARTIFACT_ROOT")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("OFFLINE SKIP"));
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
}
