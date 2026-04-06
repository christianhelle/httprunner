mod common;

use anyhow::Result;
use common::{cli::command_in, fixtures::FixtureWorkspace, server::TestServer};

#[test]
fn creates_artifacts_and_redacts_sensitive_values_by_default() -> Result<()> {
    let server = TestServer::start()?;
    let workspace = FixtureWorkspace::new(server.base_url())?;
    let output_fixture = workspace.arg("examples/outputs.local.http");

    command_in(workspace.root())
        .args([
            output_fixture.as_str(),
            "--verbose",
            "--pretty-json",
            "--log",
            "results",
            "--report",
            "--export",
            "--export-json",
            "--no-banner",
            "--no-telemetry",
        ])
        .assert()
        .success();

    let log_file = workspace.generated_file("results_", ".log")?;
    let report_file = workspace.generated_file("httprunner-report-", ".md")?;
    let json_export = workspace.generated_file("httprunner_results_", ".json")?;
    let request_export = workspace.generated_file("sensitive_payload_request_", ".log")?;
    let response_export = workspace.generated_file("sensitive_payload_response_", ".log")?;

    assert!(report_file.exists());

    let log_content = workspace.read_path(&log_file)?;
    let json_export_content = workspace.read_path(&json_export)?;
    let request_export_content = workspace.read_path(&request_export)?;
    let response_export_content = workspace.read_path(&response_export)?;

    for content in [
        log_content.as_str(),
        json_export_content.as_str(),
        request_export_content.as_str(),
        response_export_content.as_str(),
    ] {
        assert!(content.contains("***REDACTED***"));
        assert!(!content.contains("secret-token"));
        assert!(!content.contains("supersecret"));
        assert!(!content.contains("query-secret"));
    }

    Ok(())
}

#[test]
fn include_secrets_keeps_sensitive_values_in_artifacts() -> Result<()> {
    let server = TestServer::start()?;
    let workspace = FixtureWorkspace::new(server.base_url())?;
    let output_fixture = workspace.arg("examples/outputs.local.http");

    command_in(workspace.root())
        .args([
            output_fixture.as_str(),
            "--verbose",
            "--log",
            "plain-results",
            "--report",
            "html",
            "--export-json",
            "--include-secrets",
            "--no-banner",
            "--no-telemetry",
        ])
        .assert()
        .success();

    let log_file = workspace.generated_file("plain-results_", ".log")?;
    let report_file = workspace.generated_file("httprunner-report-", ".html")?;
    let json_export = workspace.generated_file("httprunner_results_", ".json")?;

    assert!(report_file.exists());

    let log_content = workspace.read_path(&log_file)?;
    let json_export_content = workspace.read_path(&json_export)?;

    assert!(log_content.contains("Bearer secret-token"));
    assert!(log_content.contains("supersecret"));
    assert!(log_content.contains("query-secret"));
    assert!(!log_content.contains("***REDACTED***"));

    assert!(json_export_content.contains("secret-token"));
    assert!(json_export_content.contains("supersecret"));
    assert!(json_export_content.contains("query-secret"));
    assert!(!json_export_content.contains("***REDACTED***"));

    Ok(())
}
