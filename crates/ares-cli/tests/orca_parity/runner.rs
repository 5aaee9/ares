//! Drives the OrcaSlicer CLI for the parity harness: builds a 3MF from
//! flattened presets and the shared cube model, slices it for the reference
//! G-code, and caches reference output keyed by the input digest.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

#[path = "runner/application.rs"]
pub(crate) mod application;
#[cfg(test)]
#[path = "runner/application_tests.rs"]
mod application_tests;
#[cfg(test)]
#[path = "runner/initialization_tests.rs"]
mod initialization_tests;
#[cfg(test)]
#[path = "runner/lifecycle_tests.rs"]
mod lifecycle_tests;
#[cfg(test)]
#[path = "runner/path_tests.rs"]
mod path_tests;
#[cfg(test)]
#[path = "runner/stage_tests.rs"]
mod stage_tests;
#[path = "runner/stages.rs"]
pub(crate) mod stages;
#[cfg(test)]
#[path = "runner/tests.rs"]
mod tests;

#[path = "runner/command.rs"]
mod command;

use stages::{FailureKind, Stage, StageError};

pub(super) struct ExportedCase {
    pub(super) label: String,
    pub(super) project: Vec<u8>,
    project_path: PathBuf,
    output_dir: PathBuf,
}

pub(super) struct OrcaRunner {
    bin: PathBuf,
    work: PathBuf,
}

pub(super) struct ParityCase {
    /// Human-readable case label (vendor / printer / variant).
    pub(super) label: String,
    pub(super) project: Vec<u8>,
    pub(super) reference: Vec<u8>,
}

/// Flattened preset inputs for one parity case.
pub(super) struct CaseInputs<'a> {
    pub(super) label: &'a str,
    pub(super) machine: &'a Map<String, Value>,
    pub(super) process: &'a Map<String, Value>,
    pub(super) filaments: &'a [Map<String, Value>],
}

impl OrcaRunner {
    /// Only absent configuration returns None; initialization errors must fail callers.
    pub(super) fn from_env() -> Result<Option<Self>, StageError> {
        let Some(bin) = std::env::var_os("ARES_ORCA_BIN")
            .map(PathBuf::from)
            .or_else(|| {
                let script = repo_root().join("scripts/orca-parity.sh");
                script.exists().then_some(script)
            })
        else {
            return Ok(None);
        };
        let init_error = |detail| StageError::new(Stage::Initialization, FailureKind::Io, detail);
        let bin = bin
            .canonicalize()
            .map_err(|error| init_error(format!("configured Orca executable {bin:?}: {error}")))?;
        let root = crate::artifacts::root_from_env().map_err(init_error)?;
        let work = tempfile::Builder::new()
            .prefix("orca-runner-")
            .tempdir_in(&root)
            .map_err(|error| init_error(format!("runner work directory under {root:?}: {error}")))?
            .keep();
        Ok(Some(Self { bin, work }))
    }

    /// Flattened preset overrides applied on top of the base presets before
    /// the 3MF export; the same map is visible to both slicers.
    pub(super) fn build_case(
        &self,
        inputs: &CaseInputs<'_>,
        overrides: &Map<String, Value>,
        model: &Path,
    ) -> Result<ParityCase, String> {
        self.build_case_checked(inputs, overrides, model)
            .map_err(|error| error.to_string())
    }

    pub(super) fn build_case_checked(
        &self,
        inputs: &CaseInputs<'_>,
        overrides: &Map<String, Value>,
        model: &Path,
    ) -> Result<ParityCase, StageError> {
        let exported = self.export_case(inputs, overrides, model)?;
        self.slice_case(exported)
    }

    pub(super) fn export_case(
        &self,
        inputs: &CaseInputs<'_>,
        overrides: &Map<String, Value>,
        model: &Path,
    ) -> Result<ExportedCase, StageError> {
        let label = inputs.label;
        let machine = apply_override(inputs.machine, overrides, PresetKind::Machine);
        let mut process = apply_override(inputs.process, overrides, PresetKind::Process);
        // The CLI's machine-switch compatibility check matches the
        // process's compatible_printers against the machine name
        // (`OrcaSlicer.cpp:2579-2585`); presets without a non-empty list
        // (e.g. the Prusa CORE One chain) fail with
        // CLI_PROCESS_NOT_COMPATIBLE (-17). Inject the machine name when
        // the process omits it or resolves it empty.
        let compatible_empty = process
            .get("compatible_printers")
            .and_then(Value::as_array)
            .is_none_or(|printers| printers.is_empty());
        if compatible_empty {
            if let Some(name) = machine.get("name").and_then(Value::as_str) {
                process.insert(
                    "compatible_printers".to_owned(),
                    Value::Array(vec![Value::String(name.to_owned())]),
                );
            }
        }
        let filaments: Vec<Map<String, Value>> = inputs
            .filaments
            .iter()
            .map(|filament| apply_override(filament, overrides, PresetKind::Filament))
            .collect();

        let slug = digest_slug(label, &machine, &process, &filaments, model);
        let project_path = self.work.join(format!("{slug}.3mf"));
        let output_dir = self.work.join(&slug);
        std::fs::create_dir_all(&output_dir)
            .map_err(|error| StageError::new(Stage::Export, FailureKind::Io, error))?;
        let model_bytes = stages::read(model, Stage::Export)?;
        std::fs::write(output_dir.join("input.stl"), model_bytes)
            .map_err(|error| StageError::new(Stage::Export, FailureKind::Io, error))?;
        if !project_path.exists() {
            let machine_file = self.work.join(format!("{slug}-machine.json"));
            let process_file = self.work.join(format!("{slug}-process.json"));
            write_preset(&machine_file, "machine", &machine)?;
            write_preset(&process_file, "process", &process)?;
            let mut filament_files = Vec::new();
            for (index, filament) in filaments.iter().enumerate() {
                let file = self.work.join(format!("{slug}-filament-{index}.json"));
                write_preset(&file, "filament", filament)?;
                filament_files.push(file);
            }

            stages::run(
                &self.bin,
                [
                    "--load-settings",
                    &join_paths([&machine_file, &process_file].into_iter()),
                    "--load-filaments",
                    &join_paths(filament_files.iter()),
                    "--arrange",
                    "1",
                    "--export-3mf",
                ]
                .into_iter()
                .chain([project_path.to_str().unwrap(), model.to_str().unwrap()])
                .map(std::borrow::ToOwned::to_owned),
                Stage::Export,
                &self.work,
            )?;
        }
        let project = stages::read(&project_path, Stage::Export)?;
        Ok(ExportedCase {
            label: label.to_owned(),
            project,
            project_path,
            output_dir,
        })
    }

    pub(super) fn slice_case(&self, exported: ExportedCase) -> Result<ParityCase, StageError> {
        let ExportedCase {
            label,
            project,
            project_path,
            output_dir,
        } = exported;
        let reference_path = output_dir.join("plate_1.gcode");
        if !reference_path.exists() {
            stages::run(
                &self.bin,
                [
                    "--slice",
                    "0",
                    "--outputdir",
                    output_dir.to_str().unwrap(),
                    project_path.to_str().unwrap(),
                ]
                .into_iter()
                .map(std::borrow::ToOwned::to_owned),
                Stage::Slice,
                &self.work,
            )?;
        }

        let reference = stages::read(&reference_path, Stage::Slice)?;
        Ok(ParityCase {
            label: label.to_owned(),
            project,
            reference,
        })
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum PresetKind {
    Machine,
    Process,
    Filament,
}

fn apply_override(
    base: &Map<String, Value>,
    overrides: &Map<String, Value>,
    kind: PresetKind,
) -> Map<String, Value> {
    let mut merged = base.clone();
    for (key, value) in overrides {
        if merged.contains_key(key) || smoke_owner(key) == Some(kind) {
            merged.insert(key.clone(), value.clone());
        }
    }
    merged
}

fn smoke_owner(key: &str) -> Option<PresetKind> {
    match key {
        "bed_exclude_area"
        | "extruder_printable_height"
        | "machine_start_gcode"
        | "retraction_distances_when_cut"
        | "use_firmware_retraction" => Some(PresetKind::Machine),
        "before_layer_change_gcode"
        | "bridge_line_width"
        | "detect_thin_wall"
        | "post_process"
        | "wall_generator" => Some(PresetKind::Process),
        _ => None,
    }
}

fn write_preset(path: &Path, kind: &str, fields: &Map<String, Value>) -> Result<(), StageError> {
    // Keep the preset's own name: the CLI matches it against
    // compatible_printers when both machine and process are loaded.
    let mut preset = fields.clone();
    preset.insert("type".into(), Value::String(kind.to_owned()));
    preset.insert("from".into(), Value::String("system".to_owned()));
    preset.insert("instantiation".into(), Value::String("true".to_owned()));
    let text = serde_json::to_string(&preset).unwrap();
    std::fs::write(path, text)
        .map_err(|e| StageError::new(Stage::Preset, FailureKind::Io, format!("{path:?}: {e}")))
}

fn join_paths<'a>(paths: impl Iterator<Item = &'a PathBuf>) -> String {
    let mut joined = String::new();
    for path in paths {
        if !joined.is_empty() {
            joined.push(';');
        }
        joined.push_str(&path.to_string_lossy());
    }
    joined
}

fn digest_slug(
    label: &str,
    machine: &Map<String, Value>,
    process: &Map<String, Value>,
    filaments: &[Map<String, Value>],
    model: &Path,
) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    label.hash(&mut hasher);
    format!("{:016x}", hasher.finish()) + &content_digest(machine, process, filaments, model)
}

fn content_digest(
    machine: &Map<String, Value>,
    process: &Map<String, Value>,
    filaments: &[Map<String, Value>],
    model: &Path,
) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    format!("{:?}", machine).hash(&mut hasher);
    format!("{:?}", process).hash(&mut hasher);
    for filament in filaments {
        format!("{:?}", filament).hash(&mut hasher);
    }
    model.to_string_lossy().hash(&mut hasher);
    format!("-{:016x}", hasher.finish())
}

pub(super) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
