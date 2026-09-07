//! External byte evidence for Print::export_gcode / GCode::do_export validation.
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub(super) const EVIDENCE: &str = "partial_semantic_only";

pub(super) fn root_from_env() -> Result<PathBuf, String> {
    let root = std::env::var_os("ARES_PARITY_ARTIFACT_ROOT")
        .map(PathBuf::from)
        .ok_or("set ARES_PARITY_ARTIFACT_ROOT to a caller-selected external directory")?;
    if !root.is_absolute() {
        return Err("ARES_PARITY_ARTIFACT_ROOT must be absolute and external".into());
    }
    let existing = root.ancestors().find(|path| path.exists()).unwrap();
    let canonical = existing.canonicalize().map_err(|e| e.to_string())?;
    let repo = crate::runner::repo_root()
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if canonical.starts_with(repo)
        || root
            .components()
            .any(|c| c == std::path::Component::ParentDir)
    {
        return Err("ARES_PARITY_ARTIFACT_ROOT must be outside the repository".into());
    }
    fs::create_dir_all(&root).map_err(|e| format!("{root:?}: {e}"))?;
    root.canonicalize().map_err(|e| e.to_string())
}

pub(super) fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{path:?}: {e}"))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("{path:?}: {e}"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn build_identity() -> Result<Value, String> {
    static IDENTITY: OnceLock<Result<Value, String>> = OnceLock::new();
    IDENTITY
        .get_or_init(|| {
            let executable = std::env::current_exe().map_err(|e| e.to_string())?;
            let mut file = fs::File::open(&executable).map_err(|e| e.to_string())?;
            let mut hasher = Sha256::new();
            let mut buffer = [0; 65536];
            loop {
                let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
                if count == 0 {
                    break;
                }
                hasher.update(&buffer[..count]);
            }
            Ok(json!({
                "executable": executable,
                "executable_sha256": hex(&hasher.finalize()),
                "revision": option_env!("ARES_PARITY_BUILD_REVISION").unwrap_or("unknown"),
                "revision_provenance": if option_env!("ARES_PARITY_BUILD_REVISION").is_some() {
                    "compile_time_environment_claim"
                } else { "unknown" },
                "generation_metadata": "2026-08-27T00:00:00",
            }))
        })
        .clone()
}

pub(super) fn compare(
    root: &Path,
    label: &str,
    project: Result<&[u8], String>,
    reference: Result<&[u8], String>,
    source: Value,
) -> Result<crate::ParityOutcome, String> {
    // Random directories prevent label/path collisions and overwriting prior evidence.
    let directory = tempfile::Builder::new()
        .prefix("case-")
        .tempdir_in(root)
        .map_err(|e| format!("{root:?}: {e}"))?
        .keep();
    eprintln!("{label}: artifacts {directory:?} ({EVIDENCE})");
    let mut files = serde_json::Map::new();
    for (name, bytes) in [("input.3mf", &project), ("orca.gcode", &reference)] {
        if let Ok(bytes) = bytes {
            save(&directory, &mut files, name, bytes)?;
        }
    }
    let mut outcome = match (project, reference) {
        (Ok(project), Ok(reference)) if !project.is_empty() && !reference.is_empty() => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .build()
                .map_err(|e| e.to_string())?;
            match runtime.block_on(ares_core::slice_project(
                project,
                ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            )) {
                Ok(actual) => {
                    // Persist BOTH full streams before the deliberately partial comparator.
                    save(&directory, &mut files, "ares.gcode", &actual)?;
                    if actual.is_empty() {
                        crate::ares_error(label, "Ares produced empty output".into())
                    } else {
                        match crate::semantic::compare_ignoring_time(reference, &actual) {
                            Ok(()) => crate::pass(label),
                            Err(difference) => crate::divergence(label, difference),
                        }
                    }
                }
                Err(error) => crate::ares_error(label, error.to_string()),
            }
        }
        (Ok(_), Ok(_)) => crate::ares_error(label, "empty project or reference bytes".into()),
        (project, reference) => crate::ares_error(
            label,
            [project.err(), reference.err()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("\n"),
        ),
    };
    if outcome.status != "PASS" {
        save(
            &directory,
            &mut files,
            "error.txt",
            outcome.detail.as_bytes(),
        )?;
    }
    let manifest = json!({
        "schema": 1,
        "label": label,
        "evidence": EVIDENCE,
        "comparator": "semantic::compare_ignoring_time",
        "not_checked": ["full XY travel", "timing", "M73", "config", "all statistics", "full command order"],
        "status": outcome.status,
        "detail": outcome.detail,
        "source": source,
        "reference_provenance": {"status": "unknown", "reason": "cached bytes do not establish producer identity"},
        "ares_build": build_identity()?,
        "files": files,
    });
    write(
        &directory.join("manifest.json"),
        &serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )?;
    outcome.artifacts = Some(directory);
    Ok(outcome)
}

fn save(
    directory: &Path,
    files: &mut serde_json::Map<String, Value>,
    name: &str,
    bytes: &[u8],
) -> Result<(), String> {
    write(&directory.join(name), bytes)?;
    files.insert(
        name.into(),
        json!({"bytes": bytes.len(), "sha256": hex(&Sha256::digest(bytes))}),
    );
    Ok(())
}
