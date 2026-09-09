use ares_core::{GenerationMetadata, slice_project};
use std::{fs, path::Path};

mod strict;

#[tokio::test]
async fn process_arachne_actual_default_prisms_full_output() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/parity/arachne");
    let project = fs::read(directory.join("default-prisms.3mf")).unwrap();
    let reference = fs::read(directory.join("default-prisms.orca.gcode")).unwrap();
    let metadata = GenerationMetadata::new_local(2026, 9, 8, 0, 0, 0).unwrap();
    let actual = slice_project(project, metadata).await.expect("ARES_ERROR");
    strict::compare(&reference, &actual).unwrap();
}
