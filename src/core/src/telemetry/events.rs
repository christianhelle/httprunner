//! Pure builders for telemetry event payloads.
//!
//! Each function maps a tracking call to its `(event_name, properties)` pair
//! without touching global state or the Application Insights client. Keeping the
//! payload shaping here makes it observable in unit tests, while `tracking::track_event`
//! remains the single (global, cfg-gated) emission point.

use super::tracking::{CliArgPatterns, ConnectionErrorCategory};
use std::collections::HashMap;

pub(super) fn cli_args_event(args: &CliArgPatterns) -> (&'static str, HashMap<String, String>) {
    let mut properties = HashMap::new();

    properties.insert("verbose".to_string(), args.verbose.to_string());
    properties.insert("log".to_string(), args.log.to_string());
    properties.insert("env".to_string(), args.env.to_string());
    properties.insert("insecure".to_string(), args.insecure.to_string());
    properties.insert(
        "include_secrets".to_string(),
        args.include_secrets.to_string(),
    );
    properties.insert("discover".to_string(), args.discover.to_string());
    properties.insert("no_banner".to_string(), args.no_banner.to_string());
    properties.insert("pretty_json".to_string(), args.pretty_json.to_string());
    properties.insert("report".to_string(), args.report.to_string());
    properties.insert("export".to_string(), args.export.to_string());
    properties.insert("export_json".to_string(), args.export_json.to_string());
    properties.insert("file_count".to_string(), args.file_count.to_string());
    properties.insert("delay".to_string(), args.delay.to_string());
    properties.insert("fail_fast".to_string(), args.fail_fast.to_string());

    if let Some(ref format) = args.report_format {
        properties.insert("report_format".to_string(), format.clone());
    }

    ("cli-args", properties)
}

pub(super) fn request_result_event(
    success: bool,
    request_count: usize,
    duration_ms: u64,
) -> (&'static str, HashMap<String, String>) {
    let mut properties = HashMap::new();
    properties.insert("success".to_string(), success.to_string());
    properties.insert("request_count".to_string(), request_count.to_string());
    properties.insert("duration_ms".to_string(), duration_ms.to_string());

    ("request-executed", properties)
}

pub(super) fn metric_event(
    metric_name: &str,
    duration_ms: u64,
    additional_props: HashMap<String, String>,
) -> (&'static str, HashMap<String, String>) {
    let mut properties = HashMap::new();
    properties.insert("metric_name".to_string(), metric_name.to_string());
    properties.insert("duration_ms".to_string(), duration_ms.to_string());

    for (key, value) in additional_props {
        properties.insert(key, value);
    }

    ("metric", properties)
}

pub(super) fn connection_error_event(
    category: ConnectionErrorCategory,
    is_insecure_mode: bool,
) -> (&'static str, HashMap<String, String>) {
    let mut properties = HashMap::new();
    properties.insert("error_category".to_string(), category.as_str().to_string());
    properties.insert("insecure_mode".to_string(), is_insecure_mode.to_string());

    ("connection-error", properties)
}

pub(super) fn feature_usage_event(
    feature_name: &str,
) -> (&'static str, HashMap<String, String>) {
    let mut properties = HashMap::new();
    properties.insert("feature_name".to_string(), feature_name.to_string());

    ("feature-used", properties)
}

pub(super) fn parse_complete_event(
    request_count: usize,
    duration_ms: u64,
) -> (&'static str, HashMap<String, String>) {
    let mut properties = HashMap::new();
    properties.insert("request_count".to_string(), request_count.to_string());
    properties.insert("duration_ms".to_string(), duration_ms.to_string());

    ("parse-complete", properties)
}

pub(super) fn execution_complete_event(
    success_count: usize,
    failed_count: usize,
    skipped_count: usize,
    total_duration_ms: u64,
) -> (&'static str, HashMap<String, String>) {
    let mut properties = HashMap::new();
    properties.insert("success_count".to_string(), success_count.to_string());
    properties.insert("failed_count".to_string(), failed_count.to_string());
    properties.insert("skipped_count".to_string(), skipped_count.to_string());
    properties.insert(
        "total_duration_ms".to_string(),
        total_duration_ms.to_string(),
    );

    ("execution-complete", properties)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_result_event_carries_outcome_properties() {
        let (name, props) = request_result_event(true, 5, 1234);
        assert_eq!(name, "request-executed");
        assert_eq!(props.get("success").map(String::as_str), Some("true"));
        assert_eq!(props.get("request_count").map(String::as_str), Some("5"));
        assert_eq!(props.get("duration_ms").map(String::as_str), Some("1234"));
    }

    #[test]
    fn metric_event_merges_additional_props() {
        let mut extra = HashMap::new();
        extra.insert("phase".to_string(), "parse".to_string());
        let (name, props) = metric_event("startup", 42, extra);
        assert_eq!(name, "metric");
        assert_eq!(props.get("metric_name").map(String::as_str), Some("startup"));
        assert_eq!(props.get("duration_ms").map(String::as_str), Some("42"));
        assert_eq!(props.get("phase").map(String::as_str), Some("parse"));
    }

    #[test]
    fn connection_error_event_uses_category_label() {
        let (name, props) = connection_error_event(ConnectionErrorCategory::Ssl, true);
        assert_eq!(name, "connection-error");
        assert_eq!(props.get("error_category").map(String::as_str), Some("ssl"));
        assert_eq!(props.get("insecure_mode").map(String::as_str), Some("true"));
    }

    #[test]
    fn feature_usage_event_carries_feature_name() {
        let (name, props) = feature_usage_event("dark-mode");
        assert_eq!(name, "feature-used");
        assert_eq!(
            props.get("feature_name").map(String::as_str),
            Some("dark-mode")
        );
    }

    #[test]
    fn parse_complete_event_carries_counts() {
        let (name, props) = parse_complete_event(7, 99);
        assert_eq!(name, "parse-complete");
        assert_eq!(props.get("request_count").map(String::as_str), Some("7"));
        assert_eq!(props.get("duration_ms").map(String::as_str), Some("99"));
    }

    #[test]
    fn execution_complete_event_carries_all_counts() {
        let (name, props) = execution_complete_event(3, 1, 2, 555);
        assert_eq!(name, "execution-complete");
        assert_eq!(props.get("success_count").map(String::as_str), Some("3"));
        assert_eq!(props.get("failed_count").map(String::as_str), Some("1"));
        assert_eq!(props.get("skipped_count").map(String::as_str), Some("2"));
        assert_eq!(
            props.get("total_duration_ms").map(String::as_str),
            Some("555")
        );
    }

    #[test]
    fn cli_args_event_includes_report_format_when_set() {
        let args = CliArgPatterns {
            verbose: true,
            file_count: 4,
            report_format: Some("json".to_string()),
            ..Default::default()
        };

        let (name, props) = cli_args_event(&args);
        assert_eq!(name, "cli-args");
        assert_eq!(props.get("verbose").map(String::as_str), Some("true"));
        assert_eq!(props.get("file_count").map(String::as_str), Some("4"));
        assert_eq!(props.get("report_format").map(String::as_str), Some("json"));
    }

    #[test]
    fn cli_args_event_omits_report_format_when_absent() {
        let args = CliArgPatterns::default();
        let (_name, props) = cli_args_event(&args);
        assert!(!props.contains_key("report_format"));
    }
}
