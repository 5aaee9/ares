//! Native test-adapter stages; no process behavior is added to ares-core.
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stage {
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
    // Print::validate does not print opt_key for -51. Generic "Too small line
    // width" is therefore NOT attributable. Only this unique bridge diagnostic is.
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
    let io_error = |error| StageError::new(stage, FailureKind::Io, error);
    let directory = tempfile::Builder::new()
        .prefix("command-")
        .tempdir_in(work)
        .map_err(io_error)?
        .keep();
    let args: Vec<String> = args.into_iter().collect();
    let environment = [
        "ORCA_APPDIR",
        "ORCA_LIB_CACHE",
        "HOME",
        "XDG_CONFIG_HOME",
        "XDG_CACHE_HOME",
        "LC_ALL",
    ]
    .into_iter()
    .map(|key| {
        (
            key,
            std::env::var_os(key).map(|value| value.to_string_lossy().into_owned()),
        )
    })
    .collect::<std::collections::BTreeMap<_, _>>();
    std::fs::write(
        directory.join("command.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "binary": bin, "args": args, "stage": format!("{stage:?}"),
            "cwd": work, "timeout_seconds": 300, "environment": environment,
        }))
        .unwrap(),
    )
    .map_err(io_error)?;
    let mut child = match Command::new(bin)
        .args(&args)
        .current_dir(work)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return finish(
                &directory,
                stage,
                Some((FailureKind::Spawn, error.to_string())),
                Captured {
                    code: None,
                    status: "not spawned".into(),
                    stdout: vec![],
                    stderr: vec![],
                },
            );
        }
    };
    let drain = |mut pipe: Box<dyn Read + Send>| {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = pipe.read_to_end(&mut bytes);
            (bytes, result)
        })
    };
    let stdout = drain(Box::new(child.stdout.take().unwrap()));
    let stderr = drain(Box::new(child.stderr.take().unwrap()));
    let deadline = Instant::now() + Duration::from_secs(300);
    let (status, mut failure) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (Some(status), None),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(25)),
            wait => {
                let reason = match wait {
                    Err(error) => (FailureKind::Wait, error.to_string()),
                    Ok(_) => (FailureKind::Timeout, "orca-slicer timed out".into()),
                };
                let kill = child.kill();
                let reaped = child.wait();
                let detail = format!("{}; kill={kill:?}; reap={reaped:?}", reason.1);
                break (reaped.ok(), Some((reason.0, detail)));
            }
        }
    };
    let mut join =
        |thread: std::thread::JoinHandle<(Vec<u8>, std::io::Result<usize>)>| match thread.join() {
            Ok((bytes, result)) => {
                if let Err(error) = result {
                    failure = Some((FailureKind::Io, error.to_string()));
                }
                bytes
            }
            Err(_) => {
                failure = Some((FailureKind::Io, "pipe drain panicked".into()));
                vec![]
            }
        };
    let stdout = join(stdout);
    let stderr = join(stderr);
    if failure.is_none() && status.is_some_and(|status| !status.success()) {
        failure = Some((FailureKind::Process, "unsuccessful Orca process".into()));
    }
    finish(
        &directory,
        stage,
        failure,
        Captured {
            code: status.and_then(|status| status.code()),
            status: status.map_or_else(|| "unavailable".into(), |status| status.to_string()),
            stdout,
            stderr,
        },
    )
}

pub(super) fn finish(
    directory: &Path,
    stage: Stage,
    failure: Option<(FailureKind, String)>,
    captured: Captured,
) -> Result<(), StageError> {
    let saved = (|| -> std::io::Result<()> {
        std::fs::write(directory.join("stdout.bin"), &captured.stdout)?;
        std::fs::write(directory.join("stderr.bin"), &captured.stderr)?;
        std::fs::write(
            directory.join("status.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "code": captured.code, "native_status": captured.status,
                "failure": failure.as_ref().map(|(kind, detail)| format!("{kind:?}: {detail}")),
            }))
            .unwrap(),
        )
    })();
    let failure = match saved {
        Ok(()) => failure,
        Err(error) => Some((FailureKind::Io, error.to_string())),
    };
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
