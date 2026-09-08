//! Actual caller entrypoints, stopped at initialization; no Orca case executes.
use std::{path::Path, process::Command};

fn invoke(selector: &str, root: Option<&Path>, cwd: &Path) -> std::process::Output {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", selector, "--nocapture"])
        .current_dir(cwd)
        .env("ARES_ORCA_BIN", std::env::current_exe().unwrap())
        .env("ARES_PARITY_SWEEP", "1")
        .env("ARES_PARITY_CLUSTER", "1")
        .env("ARES_PARITY_OPTIONS", "1")
        .env_remove("CLUSTER_DUMP")
        .env_remove("ARES_PARITY_ARTIFACT_ROOT");
    if let Some(root) = root {
        command.env("ARES_PARITY_ARTIFACT_ROOT", root);
    }
    command.output().unwrap()
}

#[test]
fn configured_callers_fail_on_artifact_initialization_errors() {
    let temp = tempfile::tempdir().unwrap();
    let blocked = temp.path().join("blocked");
    std::fs::write(&blocked, b"not a directory").unwrap();
    let repo = super::repo_root();
    let relative = std::path::PathBuf::from("relative-artifacts");
    let mut failures = Vec::new();
    for selector in [
        "smoke::orca_parity_ender3_smoke",
        "smoke::orca_parity_top_concentric_smoke",
        "smoke::orca_parity_printer_sweep",
        "smoke::orca_parity_nearest_cluster_check",
        "smoke::orca_parity_prusa_core_one_dump",
        "option_coverage::sweep::orca_parity_option_coverage",
    ] {
        for (name, root, cause) in [
            ("missing", None, "ARES_PARITY_ARTIFACT_ROOT"),
            ("relative", Some(relative.as_path()), "absolute"),
            ("repo-local", Some(repo.as_path()), "outside the repository"),
            ("unwritable", Some(blocked.as_path()), "blocked"),
        ] {
            let output = invoke(selector, root, temp.path());
            let text = format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            eprintln!("{selector}/{name}: {}\n{text}", output.status);
            if output.status.success()
                || !text.contains(cause)
                || text.contains("skipping: no OrcaSlicer")
            {
                failures.push(format!("{selector}/{name}: {}", output.status));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "initialization falsely skipped or lost cause: {failures:?}"
    );
}
