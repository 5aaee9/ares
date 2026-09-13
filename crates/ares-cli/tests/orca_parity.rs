//! OrcaSlicer parity suite: slices vendor printer profiles with both the
//! OrcaSlicer 2.4.2 CLI and Ares using the shared generator-only strict byte
//! comparator. Equality of supplied streams does not establish inventory coverage.
//!
//! Environment-gated: runs only when `ARES_ORCA_BIN` (or the repository
//! wrapper `scripts/orca-parity.sh`) names a working OrcaSlicer CLI.

#[path = "orca_parity/artifacts.rs"]
mod artifacts;
#[path = "orca_parity/m73_profile.rs"]
mod m73_profile;
#[path = "orca_parity/option_coverage.rs"]
mod option_coverage;
#[path = "orca_parity/presets.rs"]
mod presets;
#[path = "orca_parity/replay.rs"]
mod replay;
#[path = "orca_parity/replay_tests.rs"]
mod replay_tests;
#[path = "orca_parity/runner.rs"]
mod runner;
#[path = "orca_parity/smoke.rs"]
mod smoke;
#[path = "orca_parity/smoke_overrides.rs"]
mod smoke_overrides;

#[path = "ksr_fdmtest_v4/golden.rs"]
mod golden;
// Diagnostic-only semantic behavior tests; never a whole-output acceptance gate.
#[path = "ksr_fdmtest_v4/semantic.rs"]
mod semantic;

use std::collections::BTreeMap;

use presets::VendorProfiles;
use runner::{CaseInputs, OrcaRunner};
use serde_json::Value;
pub(crate) use smoke_overrides::{
    normalize_filament_defaults, normalize_process_defaults, smoke_case_overrides, smoke_overrides,
};

use crate::runner::{OrcaRunner as Runner, ParityCase};

pub(crate) struct ParityOutcome {
    pub(crate) label: String,
    pub(crate) status: &'static str,
    pub(crate) detail: String,
    pub(crate) artifacts: Option<std::path::PathBuf>,
}

pub(crate) fn compare_case(case: &ParityCase) -> ParityOutcome {
    artifacts::root_from_env()
        .and_then(|root| {
            artifacts::compare(
                &root,
                &case.label,
                Ok(&case.project),
                Ok(&case.reference),
                serde_json::json!({"kind": "runner_cache", "producer_identity": "unknown"}),
            )
        })
        .unwrap_or_else(|error| artifact_error(&case.label, error))
}

pub(crate) fn artifact_error(label: &str, error: String) -> ParityOutcome {
    ParityOutcome {
        label: label.to_owned(),
        status: "ARTIFACT_ERROR",
        detail: error,
        artifacts: None,
    }
}

pub(crate) fn pass(label: &str) -> ParityOutcome {
    ParityOutcome {
        label: label.to_owned(),
        status: "PASS",
        detail: String::new(),
        artifacts: None,
    }
}

pub(crate) fn divergence(label: &str, difference: String) -> ParityOutcome {
    ParityOutcome {
        label: label.to_owned(),
        status: "DIVERGENT",
        detail: difference,
        artifacts: None,
    }
}

pub(crate) fn ares_error(label: &str, error: String) -> ParityOutcome {
    // The upstream OrcaSlicer 2.4.2 binary itself failed (crash or nonzero
    // exit) while producing the reference stream; Ares was never invoked.
    if error.starts_with("Export/Process: unsuccessful Orca process")
        || error.starts_with("Slice/Process: unsuccessful Orca process")
    {
        return ParityOutcome {
            label: label.to_owned(),
            status: "ORCA_ERROR",
            detail: error,
            artifacts: None,
        };
    }
    // The vendor tree itself lacks the machine's referenced default preset
    // (and no compatible preset exists), so no reference producer input can
    // be assembled for this printer.
    if error.contains("default preset") && error.contains("not found")
        || error.contains("no compatible preset")
    {
        return ParityOutcome {
            label: label.to_owned(),
            status: "VENDOR_INCOMPLETE",
            detail: error,
            artifacts: None,
        };
    }
    ParityOutcome {
        label: label.to_owned(),
        status: "ARES_ERROR",
        detail: error,
        artifacts: None,
    }
}

pub(crate) struct PrinterSelection {
    pub(crate) vendor: String,
    pub(crate) printer: String,
    pub(crate) process: String,
    pub(crate) filaments: Vec<String>,
}

/// Resolves the printer's default process and filament presets, applying the
/// vendor's nozzle-variant naming ("<base> 0.4 nozzle"/"<base> 0.4").
pub(crate) fn select_printer(
    profiles: &VendorProfiles,
    vendor: &str,
    printer: &str,
) -> Result<PrinterSelection, String> {
    let machine = profiles.machine(printer)?;
    let variant = machine
        .get("printer_variant")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();

    let base_process = machine
        .get("default_print_profile")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let process = match pick_preset(
        base_process.as_deref(),
        &variant,
        |name| profiles.process_exists(name),
        &format!("{vendor}/{printer} process"),
    ) {
        Ok(process) if profiles.process_is_compatible(&process, printer) => process,
        Ok(_) => profiles
            .compatible_process(printer)
            .ok_or_else(|| format!("{vendor}/{printer} process: no compatible preset"))?,
        Err(error) => profiles.compatible_process(printer).ok_or(error)?,
    };

    let mut filaments: Vec<String> = match machine.get("default_filament_profile") {
        Some(Value::Array(names)) => names.iter().filter_map(Value::as_str).collect(),
        Some(Value::String(name)) => vec![name.as_str()],
        _ => Vec::new(),
    }
    .into_iter()
    .filter(|name| {
        !name.is_empty()
            && profiles.filament_exists(name)
            && profiles.filament_is_compatible(name, printer)
    })
    .take(1)
    .map(ToOwned::to_owned)
    .collect();
    if filaments.is_empty() {
        filaments.push(
            profiles.compatible_filament(printer).ok_or_else(|| {
                format!("{vendor}/{printer}: no usable compatible filament preset")
            })?,
        );
    }

    Ok(PrinterSelection {
        vendor: vendor.to_owned(),
        printer: printer.to_owned(),
        process,
        filaments,
    })
}

fn pick_preset(
    base: Option<&str>,
    variant: &str,
    exists: impl Fn(&str) -> bool,
    what: &str,
) -> Result<String, String> {
    let Some(base) = base.filter(|base| !base.is_empty()) else {
        return Err(format!("{what}: machine names no default preset"));
    };
    for candidate in [
        Some(format!("{base} {variant} nozzle")),
        Some(format!("{base} {variant}")),
        (!variant.is_empty()).then(|| base.to_owned()),
        Some(base.to_owned()),
    ]
    .into_iter()
    .flatten()
    {
        if exists(&candidate) {
            return Ok(candidate);
        }
    }
    Err(format!("{what}: default preset {base:?} not found"))
}

pub(crate) fn build_selection_case(
    runner: &OrcaRunner,
    profiles: &VendorProfiles,
    selection: &PrinterSelection,
    model: &std::path::Path,
) -> Result<ParityCase, String> {
    let machine = profiles.machine(&selection.printer)?;
    let mut process = profiles.process(&selection.process)?;
    normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name))
        .collect::<Result<Vec<_>, _>>()?;
    normalize_filament_defaults(&mut filaments);
    let label = format!("{}/{}", selection.vendor, selection.printer);
    let overrides = smoke_case_overrides(&machine, &process);
    runner.build_case(
        &CaseInputs {
            label: &label,
            machine: &machine,
            process: &process,
            filaments: &filaments,
        },
        &overrides,
        model,
    )
}

pub(crate) fn vendors(root: &std::path::Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut vendors = Vec::new();
    for entry in entries.flatten() {
        if let Some(name) = entry
            .path()
            .is_dir()
            .then(|| entry.file_name())
            .and_then(|name| name.to_str().map(ToOwned::to_owned))
        {
            vendors.push(name);
        }
    }
    vendors.sort();
    vendors
}

#[allow(dead_code)]
fn _type_assertions(map: &BTreeMap<String, Value>) {
    let _ = map.len();
    let _: Option<&Runner> = None;
}
