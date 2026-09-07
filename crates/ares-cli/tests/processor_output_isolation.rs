use std::{fs, path::Path};

#[path = "processor_output_isolation/mod.rs"]
mod processor_output_isolation;

use processor_output_isolation::{DUMP_VARIABLES, assert_same_gcode, slice};

const SENTINEL: &[u8] = b"; ARES_PARITY_EXTERNAL_SUBSTITUTION_SENTINEL\nG0 X987 Y654\n";

#[test]
fn processor_ignores_readable_external_gcode() {
    let temp = tempfile::tempdir().unwrap();
    let sentinel = temp.path().join("sentinel.gcode");
    fs::write(&sentinel, SENTINEL).unwrap();
    let normal = slice(temp.path(), "normal", &[]);
    let with_sentinel = slice(
        temp.path(),
        "with-sentinel",
        &[("ARES_ESTIMATE_ONLY", &sentinel)],
    );

    assert!(
        !with_sentinel
            .windows(SENTINEL.len())
            .any(|bytes| bytes == SENTINEL),
        "generated G-code was replaced by the external sentinel"
    );
    assert_same_gcode(&normal, &with_sentinel);
    assert_eq!(fs::read(sentinel).unwrap(), SENTINEL);
}

#[test]
fn processor_ignores_missing_external_gcode() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing.gcode");
    let normal = slice(temp.path(), "normal", &[]);
    let with_missing = slice(
        temp.path(),
        "with-missing",
        &[("ARES_ESTIMATE_ONLY", &missing)],
    );

    assert_same_gcode(&normal, &with_missing);
    assert!(!missing.exists());
}

#[test]
fn processor_ignores_existing_unreadable_gcode_path() {
    let temp = tempfile::tempdir().unwrap();
    // A directory cannot be read as a G-code file on any Tier 1 native OS;
    // unlike permission bits, this also exercises the case when run as root.
    let directory = temp.path().join("directory.gcode");
    fs::create_dir(&directory).unwrap();
    let normal = slice(temp.path(), "normal", &[]);
    let with_directory = slice(
        temp.path(),
        "with-directory",
        &[("ARES_ESTIMATE_ONLY", &directory)],
    );

    assert_same_gcode(&normal, &with_directory);
    assert_eq!(fs::read_dir(directory).unwrap().count(), 0);
}

#[test]
fn processor_dump_variables_neither_create_nor_modify_files() {
    let temp = tempfile::tempdir().unwrap();
    let paths = DUMP_VARIABLES.map(|name| temp.path().join(name));
    let variables = DUMP_VARIABLES
        .iter()
        .zip(&paths)
        .map(|(&name, path)| (name, path.as_path()))
        .collect::<Vec<(&str, &Path)>>();
    let normal = slice(temp.path(), "normal", &[]);
    let with_missing_dumps = slice(temp.path(), "missing-dumps", &variables);

    assert_same_gcode(&normal, &with_missing_dumps);
    for path in &paths {
        assert!(!path.exists(), "processor created {}", path.display());
        fs::write(path, SENTINEL).unwrap();
    }
    let with_existing_dumps = slice(temp.path(), "existing-dumps", &variables);
    assert_same_gcode(&normal, &with_existing_dumps);
    for path in &paths {
        assert_eq!(fs::read(path).unwrap(), SENTINEL);
    }
}
