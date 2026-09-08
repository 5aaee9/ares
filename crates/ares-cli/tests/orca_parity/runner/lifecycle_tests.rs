//! Short-lived helpers exercise the actual command lifecycle, never Orca success.
use super::stages::{Captured, FailureKind, Stage, finish, run_with_timeout};
use std::{
    io::Write,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[test]
fn timeout_returns_before_descendant_closes_inherited_output_handles() {
    let temp = tempfile::Builder::new()
        .prefix("lifecycle-")
        .tempdir()
        .unwrap()
        .keep();
    eprintln!("retained helper artifacts: {temp:?}");
    let descendant = temp.join("descendant");
    std::fs::create_dir(&descendant).unwrap();
    std::fs::write(temp.join("helper-mode"), "parent").unwrap();
    std::fs::write(descendant.join("helper-mode"), "descendant").unwrap();
    let start = Instant::now();
    let error = run_with_timeout(
        &std::env::current_exe().unwrap(),
        [
            "--exact",
            "runner::lifecycle_tests::bounded_output_holder",
            "--nocapture",
        ]
        .map(str::to_owned),
        Stage::Export,
        &temp,
        Duration::from_millis(500),
    )
    .unwrap_err();
    let elapsed = start.elapsed();
    let descendant_still_running = !descendant.join("finished").exists();
    let directory = error.evidence.as_ref().unwrap();
    let stdout = std::fs::read(directory.join("stdout.bin")).unwrap();
    let stderr = std::fs::read(directory.join("stderr.bin")).unwrap();
    // Helpers always end independently; wait for their marker before asserting,
    // including on the old implementation's expected-red path.
    let cleanup = Instant::now() + Duration::from_secs(5);
    while !descendant.join("finished").exists() && Instant::now() < cleanup {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        descendant.join("finished").exists(),
        "bounded descendant did not finish"
    );
    eprintln!("elapsed={elapsed:?}, error={error:?}");
    assert!(
        descendant_still_running && elapsed < Duration::from_secs(2),
        "waited for inherited output EOF: {elapsed:?}"
    );
    assert_eq!(error.kind, FailureKind::Timeout);
    assert!(error.detail.contains("kill=") && error.detail.contains("reap="));
    assert_ne!(error.captured.as_ref().unwrap().status, "unavailable");
    assert!(
        stdout
            .windows(b"stdout-before-timeout\0\xff".len())
            .any(|b| b == b"stdout-before-timeout\0\xff")
    );
    assert!(
        stderr
            .windows(b"stderr-before-timeout\0\xff".len())
            .any(|b| b == b"stderr-before-timeout\0\xff")
    );
    assert_eq!(std::fs::read(directory.join("stdout.bin")).unwrap(), stdout);
    assert_eq!(std::fs::read(directory.join("stderr.bin")).unwrap(), stderr);
    assert!(!error.input_rejection("line_width"));
    let status: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("status.json")).unwrap()).unwrap();
    let lifecycle: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("lifecycle.json")).unwrap()).unwrap();
    assert_eq!(status["snapshot_incomplete"], true);
    assert_eq!(lifecycle["direct_child_reaped"], true);
    assert!(
        status["live_captures"]
            .as_str()
            .unwrap()
            .contains("non-authoritative")
    );
    assert!(
        String::from_utf8_lossy(&std::fs::read(directory.join("stdout.live")).unwrap())
            .contains("descendant-late-output")
    );
    assert!(!String::from_utf8_lossy(&stdout).contains("descendant-late-output"));
}

#[test]
fn bounded_output_holder() {
    let Ok(mode) = std::fs::read_to_string("helper-mode") else {
        return;
    };
    if mode == "descendant" {
        std::thread::sleep(Duration::from_secs(3));
        println!("descendant-late-output");
        std::fs::write("finished", b"done").unwrap();
        return;
    }
    let mut descendant = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "runner::lifecycle_tests::bounded_output_holder",
            "--nocapture",
        ])
        .current_dir("descendant")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    std::io::stdout()
        .write_all(b"stdout-before-timeout\0\xff")
        .unwrap();
    std::io::stdout().flush().unwrap();
    std::io::stderr()
        .write_all(b"stderr-before-timeout\0\xff")
        .unwrap();
    std::io::stderr().flush().unwrap();
    std::fs::write("direct-child.pid", std::process::id().to_string()).unwrap();
    descendant.wait().unwrap();
    std::thread::sleep(Duration::from_secs(1));
}

#[test]
fn capture_persistence_error_preserves_timeout_or_wait_cause() {
    for kind in [FailureKind::Timeout, FailureKind::Wait] {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp.path().join("status.json")).unwrap();
        let error = finish(
            temp.path(),
            Stage::Export,
            Some((kind, "primary lifecycle failure".into())),
            Captured {
                code: None,
                status: "reaped".into(),
                stdout: b"captured stdout".to_vec(),
                stderr: b"captured stderr".to_vec(),
            },
        )
        .unwrap_err();
        assert_eq!(error.kind, kind);
        assert!(error.detail.contains("primary lifecycle failure"));
        assert!(!error.input_rejection("line_width"));
    }
}
