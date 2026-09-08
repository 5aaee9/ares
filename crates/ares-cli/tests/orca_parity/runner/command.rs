//! Portable file-backed command capture: never wait for inherited-handle EOF.
use super::stages::{Captured, FailureKind, Stage, StageError, capture_error, finish};
use std::{
    fs::File,
    io::Read,
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

pub(super) fn run(
    bin: &Path,
    args: impl IntoIterator<Item = String>,
    stage: Stage,
    work: &Path,
    timeout: Duration,
) -> Result<(), StageError> {
    let io_error = |error| StageError::new(stage, FailureKind::Io, error);
    let requested_bin = bin;
    let bin = std::path::absolute(bin).map_err(io_error)?;
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
    std::fs::write(directory.join("command.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "binary": bin, "requested_binary": requested_bin, "args": args, "stage": format!("{stage:?}"),
        "cwd": work, "timeout_millis": timeout.as_millis(), "shutdown_grace_millis": 2000,
        "environment": environment,
        "capture_policy": "file-backed; immutable length-bounded snapshots; no inherited-pipe EOF wait",
        "process_tree_termination": "not guaranteed",
    })).unwrap()).map_err(io_error)?;
    let stdout_path = directory.join("stdout.live");
    let stderr_path = directory.join("stderr.live");
    let stdout_file = File::create(&stdout_path).map_err(io_error)?;
    let stderr_file = File::create(&stderr_path).map_err(io_error)?;
    let mut child = match Command::new(&bin)
        .args(&args)
        .current_dir(work)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
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
    let pid = child.id();
    let deadline = Instant::now() + timeout;
    let mut failure = None;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            result => {
                let reason = match result {
                    Err(error) => (FailureKind::Wait, error.to_string()),
                    Ok(_) => (FailureKind::Timeout, "orca-slicer timed out".into()),
                };
                let (status, shutdown) = shutdown(&mut child);
                failure = Some((reason.0, format!("{}; {shutdown}", reason.1)));
                break status;
            }
        }
    };
    if failure.is_none() && status.is_some_and(|status| !status.success()) {
        failure = Some((FailureKind::Process, "unsuccessful Orca process".into()));
    }
    let lifecycle = serde_json::json!({"direct_child_pid": pid, "direct_child_reaped": status.is_some(),
        "native_status": status.map(|status| status.to_string()), "process_tree_termination": "not guaranteed"});
    if let Err(error) = std::fs::write(
        directory.join("lifecycle.json"),
        serde_json::to_vec_pretty(&lifecycle).unwrap(),
    ) {
        capture_error(&mut failure, error.to_string());
    }
    let (stdout, stdout_error) = snapshot(&stdout_path);
    let (stderr, stderr_error) = snapshot(&stderr_path);
    for error in [stdout_error, stderr_error].into_iter().flatten() {
        capture_error(&mut failure, error);
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

fn shutdown(child: &mut Child) -> (Option<ExitStatus>, String) {
    let kill = child.kill();
    let deadline = Instant::now() + Duration::from_secs(2);
    let reap = loop {
        match child.try_wait() {
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            result => break result,
        }
    };
    let detail = format!("kill={kill:?}; reap={reap:?}");
    // None/error is an explicitly unreaped child, never a successful stage.
    (reap.ok().flatten(), detail)
}

fn snapshot(path: &Path) -> (Vec<u8>, Option<String>) {
    let mut bytes = Vec::new();
    let result = (|| -> std::io::Result<()> {
        let file = File::open(path)?;
        let length = file.metadata()?.len();
        file.take(length).read_to_end(&mut bytes)?;
        Ok(())
    })();
    (
        bytes,
        result
            .err()
            .map(|error| format!("snapshot {path:?}: {error}")),
    )
}
