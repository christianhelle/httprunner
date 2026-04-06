mod common;

use anyhow::Result;
use common::{cli::command_in, fixtures::FixtureWorkspace, server::TestServer};
use predicates::prelude::*;

#[test]
fn no_args_prints_help_and_exits_successfully() {
    let mut cmd = assert_cmd::Command::cargo_bin("httprunner").unwrap();
    cmd.assert().success().stdout(
        predicate::str::contains("Usage:").or(predicate::str::contains("HTTP File Runner")),
    );
}

#[test]
fn runs_single_and_multiple_local_example_files() -> Result<()> {
    let server = TestServer::start()?;
    let workspace = FixtureWorkspace::new(server.base_url())?;
    let basic_fixture = workspace.arg("examples/basic.local.http");
    let intellij_fixture = workspace.arg("examples/intellij.local.http");

    command_in(workspace.root())
        .args([
            basic_fixture.as_str(),
            intellij_fixture.as_str(),
            "--no-banner",
            "--no-telemetry",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "All discovered files processed successfully",
        ))
        .stdout(predicate::str::contains("Buy me a coffee").not())
        .stdout(predicate::str::contains("Support Key").not());

    Ok(())
}

#[test]
fn discover_mode_runs_nested_fixture_tree() -> Result<()> {
    let server = TestServer::start()?;
    let workspace = FixtureWorkspace::new(server.base_url())?;

    command_in(&workspace.path("discover"))
        .args(["--discover", "--no-banner", "--no-telemetry"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 2 .http file(s):"))
        .stdout(predicate::str::contains("root.local.http"))
        .stdout(predicate::str::contains("child.local.http"))
        .stdout(predicate::str::contains("Support Key").not());

    Ok(())
}
