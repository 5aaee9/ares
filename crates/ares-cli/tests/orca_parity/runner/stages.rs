//! Native test-adapter stages; no process behavior is added to ares-core.
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stage {
    Initialization,
    Preset,
    Export,
    Slice,
    Application,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FailureKind {
    Spawn,
    Wait,
    Timeout,
    Io,
    MissingArtifact,
    Process,
    ApplicationMismatch,
}

#[derive(Debug)]
pub(crate) struct Captured {
    pub(crate) code: Option<i32>,
    pub(crate) status: String,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct StageError {
    pub(crate) stage: Stage,
    pub(crate) kind: FailureKind,
    pub(crate) detail: String,
    pub(crate) evidence: Option<PathBuf>,
    pub(crate) captured: Option<Box<Captured>>,
}

impl StageError {
    pub(crate) fn new(stage: Stage, kind: FailureKind, detail: impl ToString) -> Self {
        Self {
            stage,
            kind,
            detail: detail.to_string(),
            evidence: None,
            captured: None,
        }
    }

    pub(crate) fn input_rejection(&self, key: &str) -> bool {
        self.kind == FailureKind::Process
            && self
                .captured
                .as_ref()
                .is_some_and(|output| attributable(output, key))
    }
}

impl std::fmt::Display for StageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}/{:?}: {}; evidence {:?}",
            self.stage, self.kind, self.detail, self.evidence
        )
    }
}

pub(crate) fn attributable(output: &Captured, key: &str) -> bool {
    // Utils.hpp CLI codes; Unix retains the low 8 bits, Windows retains signed i32.
    let code = |value: i32| {
        if cfg!(unix) {
            value.rem_euclid(256)
        } else {
            value
        }
    };
    let text = [
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    ]
    .join("\n");
    if output.code == Some(code(-18)) {
        return text.contains("Param values in 3mf/config error:")
            && text
                .lines()
                .any(|line| line.starts_with(&format!("{key}: ")));
    }
    // Print::validate omits opt_key for -51. Generic "Too small line width"
    // is not attributable; only this unique bridge diagnostic qualifies.
    output.code == Some(code(-51))
        && key == "bridge_line_width"
        && text
            .lines()
            .any(|line| line == "Bridge line width must not exceed nozzle diameter")
}

pub(crate) fn run(
    bin: &Path,
    args: impl IntoIterator<Item = String>,
    stage: Stage,
    work: &Path,
) -> Result<(), StageError> {
    run_with_timeout(bin, args, stage, work, Duration::from_secs(300))
}

pub(super) fn run_with_timeout(
    bin: &Path,
    args: impl IntoIterator<Item = String>,
    stage: Stage,
    work: &Path,
    timeout: Duration,
) -> Result<(), StageError> {
    super::command::run(bin, args, stage, work, timeout)
}

pub(super) fn capture_error(failure: &mut Option<(FailureKind, String)>, detail: String) {
    match failure {
        Some((FailureKind::Timeout | FailureKind::Wait, primary)) => {
            primary.push_str(&format!("; capture I/O error: {detail}"));
        }
        _ => *failure = Some((FailureKind::Io, detail)),
    }
}

pub(super) fn finish(
    directory: &Path,
    stage: Stage,
    mut failure: Option<(FailureKind, String)>,
    captured: Captured,
) -> Result<(), StageError> {
    for (name, bytes) in [
        ("stdout.bin", &captured.stdout),
        ("stderr.bin", &captured.stderr),
    ] {
        if let Err(error) = crate::artifacts::write(&directory.join(name), bytes) {
            capture_error(&mut failure, error);
        }
    }
    let hash = |bytes: &[u8]| {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let status = serde_json::json!({
        "code": captured.code, "native_status": captured.status,
        "failure": failure.as_ref().map(|(kind, detail)| format!("{kind:?}: {detail}")),
        "snapshot_scope": "direct child output only; not process-tree completion",
        "snapshot_incomplete": failure.as_ref().is_some_and(|(kind, _)| *kind != FailureKind::Process),
        "live_captures": "stdout.live/stderr.live are non-authoritative and may continue changing after return",
        "stdout": {"file": "stdout.bin", "bytes": captured.stdout.len(), "sha256": hash(&captured.stdout)},
        "stderr": {"file": "stderr.bin", "bytes": captured.stderr.len(), "sha256": hash(&captured.stderr)},
    });
    if let Err(error) = crate::artifacts::write(
        &directory.join("status.json"),
        &serde_json::to_vec_pretty(&status).unwrap(),
    ) {
        capture_error(&mut failure, error);
    }
    if let Some((kind, detail)) = failure {
        Err(StageError {
            stage,
            kind,
            detail,
            evidence: Some(directory.to_owned()),
            captured: Some(Box::new(captured)),
        })
    } else {
        Ok(())
    }
}

pub(crate) fn read(path: &Path, stage: Stage) -> Result<Vec<u8>, StageError> {
    std::fs::read(path).map_err(|error| {
        StageError::new(
            stage,
            if error.kind() == std::io::ErrorKind::NotFound {
                FailureKind::MissingArtifact
            } else {
                FailureKind::Io
            },
            format!("{path:?}: {error}"),
        )
    })
}
