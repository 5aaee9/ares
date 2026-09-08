use super::stages::{Captured, FailureKind, Stage, StageError, attributable, finish};

fn output(code: Option<i32>, stderr: &str) -> Captured {
    Captured {
        code: code.map(|code| {
            if cfg!(unix) {
                code.rem_euclid(256)
            } else {
                code
            }
        }),
        status: format!("captured {code:?}"),
        stdout: b"full stdout\0\xff".to_vec(),
        stderr: stderr.as_bytes().to_vec(),
    }
}

#[test]
fn validation_status_requires_source_attributable_diagnostic() {
    for (code, diagnostic, key, expected) in [
        (
            Some(-18),
            "Param values in 3mf/config error:\nline_width: Value out of range",
            "line_width",
            true,
        ),
        (
            Some(-18),
            "Param values in 3mf/config error:\nbridge_line_width: error",
            "line_width",
            false,
        ),
        (Some(-18), "line_width: error", "line_width", false),
        (
            Some(-5),
            "Param values in 3mf/config error:\nline_width: error",
            "line_width",
            false,
        ),
        (Some(-51), "Too small line width", "line_width", false),
        (
            Some(-51),
            "Bridge line width must not exceed nozzle diameter",
            "bridge_line_width",
            true,
        ),
        (
            Some(-51),
            "Bridge line width must not exceed nozzle diameter",
            "line_width",
            false,
        ),
        (
            Some(0),
            "Param values in 3mf/config error:\nline_width: error",
            "line_width",
            false,
        ),
        (
            None,
            "Param values in 3mf/config error:\nline_width: error",
            "line_width",
            false,
        ),
    ] {
        assert_eq!(
            attributable(&output(code, diagnostic), key),
            expected,
            "{code:?} {diagnostic}"
        );
    }
}

#[test]
fn infrastructure_failures_never_become_input_rejections() {
    for kind in [
        FailureKind::Spawn,
        FailureKind::Wait,
        FailureKind::Timeout,
        FailureKind::Io,
        FailureKind::MissingArtifact,
        FailureKind::ApplicationMismatch,
    ] {
        let error = StageError {
            stage: Stage::Export,
            kind,
            detail: "bounded captured result".into(),
            evidence: None,
            captured: Some(Box::new(output(
                Some(-18),
                "Param values in 3mf/config error:\nline_width: error",
            ))),
        };
        assert!(!error.input_rejection("line_width"), "{kind:?}");
    }
}

#[test]
fn completed_and_failed_stage_results_retain_full_binary_output_and_status() {
    for kind in [
        None,
        Some(FailureKind::Process),
        Some(FailureKind::Timeout),
        Some(FailureKind::Wait),
        Some(FailureKind::Spawn),
    ] {
        let root = tempfile::tempdir().unwrap();
        let captured = output(
            Some(if kind.is_some() { -18 } else { 0 }),
            "first line\nkey: error\nlast line\n",
        );
        let status = captured.status.clone();
        let stdout = captured.stdout.clone();
        let stderr = captured.stderr.clone();
        let result = finish(
            root.path(),
            Stage::Slice,
            kind.map(|kind| (kind, "failure detail".into())),
            captured,
        );
        assert_eq!(result.is_err(), kind.is_some());
        assert_eq!(
            std::fs::read(root.path().join("stdout.bin")).unwrap(),
            stdout
        );
        assert_eq!(
            std::fs::read(root.path().join("stderr.bin")).unwrap(),
            stderr
        );
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.path().join("status.json")).unwrap())
                .unwrap();
        assert_eq!(saved["native_status"], status);
        if let Err(error) = result {
            assert_eq!(error.kind, kind.unwrap());
        }
    }
}

#[test]
fn missing_artifact_is_not_input_rejection() {
    let root = tempfile::tempdir().unwrap();
    let error = super::stages::read(&root.path().join("absent.gcode"), Stage::Slice).unwrap_err();
    assert_eq!(error.kind, FailureKind::MissingArtifact);
    assert!(!error.input_rejection("line_width"));
}
