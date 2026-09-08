use std::{path::Path, process::Command};

#[test]
fn relative_executable_survives_changed_command_cwd() {
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("command-work");
    std::fs::create_dir(&work).unwrap();
    std::fs::copy(
        std::env::current_exe().unwrap(),
        temp.path().join("runner-helper.exe"),
    )
    .unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "runner::path_tests::relative_launch_probe",
            "--nocapture",
        ])
        .env("ARES_RUNNER_PATH_WORK", &work)
        .env("ARES_ORCA_BIN", "./runner-helper.exe")
        .env(
            "ARES_PARITY_ARTIFACT_ROOT",
            temp.path().join("runner-artifacts"),
        )
        .current_dir(temp.path())
        .output()
        .unwrap();
    eprintln!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "relative launch: {}",
        output.status
    );
}

#[test]
fn relative_launch_probe() {
    let Some(work) = std::env::var_os("ARES_RUNNER_PATH_WORK") else {
        return;
    };
    let work = Path::new(&work);
    let runner = super::OrcaRunner::from_env().unwrap().unwrap();
    assert!(runner.bin.is_absolute());
    assert_eq!(
        runner.bin,
        Path::new("./runner-helper.exe").canonicalize().unwrap()
    );
    super::stages::run(
        Path::new("./runner-helper.exe"),
        [
            "--exact",
            "runner::path_tests::bounded_launch_helper",
            "--nocapture",
        ]
        .map(str::to_owned),
        super::stages::Stage::Export,
        work,
    )
    .unwrap();
    let directory = std::fs::read_dir(work)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let command: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.join("command.json")).unwrap()).unwrap();
    let binary = Path::new(command["binary"].as_str().unwrap());
    assert!(binary.is_absolute());
    assert_eq!(
        binary.canonicalize().unwrap(),
        Path::new("./runner-helper.exe").canonicalize().unwrap()
    );
    assert!(
        String::from_utf8_lossy(&std::fs::read(directory.join("stdout.bin")).unwrap())
            .contains("bounded-path-helper")
    );
}

#[test]
fn bounded_launch_helper() {
    println!("bounded-path-helper");
}
