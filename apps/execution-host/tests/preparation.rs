mod support;
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};
use support::*;

fn refused(project: &Path, id: &str, run: &Path, software: bool) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_stagemaster-execution-host"));
    command
        .arg(project)
        .args(["sequence", id])
        .arg(run)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if software {
        command.arg("--software-output");
    }
    assert!(!command.status().unwrap().success());
    assert!(!run.join("discovery.json").exists());
}
#[test]
fn startup_refuses_bad_sources_wrong_selection_existing_directory_and_implicit_mode() {
    let dir = temporary();
    let show = dir.path().join("show.json");
    project(&show);
    let original = fs::read(&show).unwrap();
    refused(&show, SEQUENCE, &dir.path().join("implicit"), false);
    refused(
        &show,
        "00000000-0000-4000-8000-000000000099",
        &dir.path().join("missing-program"),
        true,
    );
    let existing = dir.path().join("existing");
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("preserve"), b"untouched").unwrap();
    refused(&show, SEQUENCE, &existing, true);
    assert_eq!(fs::read(existing.join("preserve")).unwrap(), b"untouched");
    assert_eq!(fs::read(&show).unwrap(), original);
    fs::write(&show, b"{}").unwrap();
    refused(&show, SEQUENCE, &dir.path().join("invalid-project"), true);
    refused(
        &dir.path().join("missing.json"),
        SEQUENCE,
        &dir.path().join("missing-file"),
        true,
    );
}
#[test]
fn prepared_content_is_independent_of_source_file_and_credentials_are_private() {
    runtime().block_on(
        prepared_content_is_independent_of_source_file_and_credentials_are_private_async(),
    );
}
async fn prepared_content_is_independent_of_source_file_and_credentials_are_private_async() {
    let mut h = Harness::start();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [
            h.directory.path().join("run"),
            h.directory.path().join("run/discovery.json"),
        ] {
            assert_eq!(fs::metadata(path).unwrap().permissions().mode() & 0o077, 0);
        }
    }
    let source = ok(h.get("/source")).await;
    fs::remove_file(&h.project).unwrap();
    let id = h.session().await;
    let acquired = h.acquire(&id, false).await;
    let started = h
        .start_program(&id, &acquired["state"]["revision"], 2)
        .await;
    assert_eq!(started["kind"], "applied");
    assert_eq!(ok(h.get("/source")).await, source);
    fs::write(&h.project, &h.original).unwrap();
    h.close().await;
}
