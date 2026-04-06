mod common;

use anyhow::Result;
use common::{cli::command_in, fixtures::FixtureWorkspace, server::TestServer};
use predicates::prelude::*;
use std::time::{Duration, Instant};

#[test]
fn env_fixture_uses_http_client_env_file() -> Result<()> {
    let server = TestServer::start()?;
    let workspace = FixtureWorkspace::new(server.base_url())?;
    let env_fixture = workspace.arg("env/env.local.http");

    command_in(workspace.root())
        .args([
            env_fixture.as_str(),
            "--env",
            "local",
            "--no-banner",
            "--no-telemetry",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "All discovered files processed successfully",
        ))
        .stdout(predicate::str::contains("Support Key").not());

    Ok(())
}

#[test]
fn feature_fixture_covers_chaining_conditionals_and_delay() -> Result<()> {
    let server = TestServer::start()?;
    let workspace = FixtureWorkspace::new(server.base_url())?;
    let feature_fixture = workspace.arg("examples/features.local.http");
    let start = Instant::now();

    command_in(workspace.root())
        .args([
            feature_fixture.as_str(),
            "--delay",
            "20",
            "--verbose",
            "--pretty-json",
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

    assert!(
        start.elapsed() >= Duration::from_millis(150),
        "expected CLI delay handling to add noticeable execution time"
    );

    Ok(())
}
