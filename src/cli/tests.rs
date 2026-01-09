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
