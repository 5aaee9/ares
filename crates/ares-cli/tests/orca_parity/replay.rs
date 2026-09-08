//! Strict comparison of cached project/reference streams, not a fresh Orca execution.
//! Explicit requests require ARES_PARITY_REPLAY and external ARES_PARITY_ARTIFACT_ROOT.
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::{self as parity, artifacts};

struct ReplayCase {
    label: String,
    project: PathBuf,
    reference: PathBuf,
}

fn replay_one(case: &ReplayCase, root: &Path) -> parity::ParityOutcome {
    let project = std::fs::read(&case.project)
        .map_err(|error| format!("fixture 3mf {:?}: {error}", case.project));
    let reference = std::fs::read(&case.reference)
        .map_err(|error| format!("reference gcode {:?}: {error}", case.reference));
    artifacts::compare(
        root,
        &case.label,
        project.as_deref().map_err(Clone::clone),
        reference.as_deref().map_err(Clone::clone),
        json!({"kind": "legacy_replay", "project": case.project, "reference": case.reference}),
    )
    .unwrap_or_else(|error| parity::artifact_error(&case.label, error))
}

#[test]
fn orca_parity_replay_sweep() {
    let Some(root) = std::env::var_os("ARES_PARITY_REPLAY") else {
        eprintln!("OFFLINE SKIP: ARES_PARITY_REPLAY unset; no parity comparison executed");
        return;
    };
    run(&PathBuf::from(root)).unwrap_or_else(|error| panic!("replay failed: {error}"));
}

fn run(input: &Path) -> Result<(), String> {
    let root = artifacts::root_from_env()?;
    let mut cases = Vec::new();
    for entry in std::fs::read_dir(input).map_err(|e| format!("{input:?}: {e}"))? {
        let path = entry
            .map_err(|e| format!("{input:?} directory entry: {e}"))?
            .path();
        if path.extension().is_none_or(|extension| extension != "3mf") {
            continue;
        }
        let stem = path.file_stem().unwrap();
        let label = stem
            .to_str()
            .ok_or_else(|| format!("non-UTF8 case label: {path:?}"))?
            .to_owned();
        cases.push(ReplayCase {
            label,
            reference: path.parent().unwrap().join(stem).join("plate_1.gcode"),
            project: path,
        });
    }
    cases.sort_by(|a, b| a.label.cmp(&b.label));
    if cases.is_empty() {
        return Err(format!("empty replay inventory: {input:?}"));
    }
    eprintln!(
        "replaying {} cached fixtures; {}",
        cases.len(),
        artifacts::EVIDENCE
    );
    let workers = std::thread::available_parallelism()
        .map_or(1, |value| value.get())
        .min(cases.len());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let worker = || {
        let mut done = Vec::new();
        loop {
            let index = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let Some(case) = cases.get(index) else {
                return done;
            };
            done.push(replay_one(case, &root));
        }
    };
    let mut outcomes: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers).map(|_| scope.spawn(worker)).collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect()
    });
    outcomes.sort_by(|a, b| a.label.cmp(&b.label));
    let passing = outcomes
        .iter()
        .filter(|outcome| outcome.status == "PASS")
        .count();
    let compared = outcomes
        .iter()
        .filter(|outcome| matches!(outcome.status, "PASS" | "DIVERGENT"))
        .count();
    let summary = json!({
        "evidence": artifacts::EVIDENCE,
        "input": input,
        "inventory": cases.len(),
        "compared": compared,
        "passed": passing,
        "failed": cases.len() - passing,
        "cases": outcomes.iter().map(|outcome| json!({
            "label": outcome.label, "status": outcome.status,
            "detail": outcome.detail, "artifacts": outcome.artifacts,
        })).collect::<Vec<_>>(),
    });
    let report = root.join("replay-summary.json");
    artifacts::write(
        &report,
        &serde_json::to_vec_pretty(&summary).map_err(|e| e.to_string())?,
    )?;
    eprintln!(
        "ordered byte replay (generator only): {passing}/{} passed; report {report:?}",
        cases.len()
    );
    if passing != cases.len() {
        return Err(format!(
            "{} failed cases; supplied streams do not match; see {report:?}",
            cases.len() - passing
        ));
    }
    Ok(())
}
