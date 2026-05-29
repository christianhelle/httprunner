use super::args::*;

fn cli_with_log(log: Option<Option<&str>>) -> Cli {
    Cli {
        files: vec![],
        verbose: false,
        log: log.map(|opt| opt.map(|s| s.to_string())),
        env: None,
        insecure: false,
        discover: false,
        upgrade: false,
        no_banner: false,
        pretty_json: false,
        report: None,
        export: false,
        export_json: false,
        include_secrets: false,
        no_telemetry: false,
        delay: 0,
        fail_fast: false,
    }
}

#[test]
fn get_log_filename_returns_explicit_value() {
    let cli = cli_with_log(Some(Some("custom")));
    assert_eq!(cli.get_log_filename().as_deref(), Some("custom"));
}

#[test]
fn get_log_filename_defaults_to_log_name() {
    let cli = cli_with_log(Some(None));
    assert_eq!(cli.get_log_filename().as_deref(), Some("log"));
}

#[test]
fn get_log_filename_none_when_flag_missing() {
    let cli = cli_with_log(None);
    assert!(cli.get_log_filename().is_none());
}

#[test]
fn fail_fast_flag_defaults_to_false() {
    use clap::Parser;
    let cli = Cli::try_parse_from(["httprunner", "test.http"]).unwrap();
    assert!(!cli.fail_fast);
}

#[test]
fn fail_fast_flag_parses_long_form() {
    use clap::Parser;
    let cli = Cli::try_parse_from(["httprunner", "--fail-fast", "test.http"]).unwrap();
    assert!(cli.fail_fast);
}

#[test]
fn show_donation_banner_outputs_message() {
    // This test simply ensures show_donation_banner runs without panic
    // We can't easily capture stdout without more complex testing infrastructure,
    // but we can at least ensure the function executes
    use super::banner::show_donation_banner;
    show_donation_banner();
}
