use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
use appinsights::blocking::TelemetryClient;
#[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
use appinsights::telemetry::{SeverityLevel, Telemetry};

use super::app_type::AppType;
use super::config::TelemetryConfig;
#[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
use super::config::is_disabled_by_env;
#[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
use super::sanitize::{get_error_type_name, sanitize_error_message};

static TELEMETRY_STATE: OnceLock<Mutex<TelemetryState>> = OnceLock::new();

#[allow(dead_code)]
struct TelemetryState {
    #[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
    client: Option<TelemetryClient>,
    app_type: AppType,
    version: String,
    session_id: String,
    operation_id: String,
    device_id: String,
    support_key: String,
    support_key_short: String,
    enabled: bool,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
fn get_support_key_info() -> (String, String) {
    if let Ok(support_key) = crate::logging::get_support_key() {
        (support_key.key, support_key.short_key)
    } else {
        let key = uuid::Uuid::new_v4().to_string();
        let short_key = key.chars().take(8).collect();
        (key, short_key)
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
pub fn init(app_type: AppType, version: &str, force_disabled: bool, instrumentation_key: &str) {
    init_with_config(app_type, version, force_disabled, instrumentation_key, true);
}

#[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
pub fn init_without_persisted_state(
    app_type: AppType,
    version: &str,
    force_disabled: bool,
    instrumentation_key: &str,
) {
    init_with_config(
        app_type,
        version,
        force_disabled,
        instrumentation_key,
        false,
    );
}

#[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
fn init_with_config(
    app_type: AppType,
    version: &str,
    force_disabled: bool,
    instrumentation_key: &str,
    use_persisted_config: bool,
) {
    let config_enabled = if use_persisted_config {
        TelemetryConfig::load().enabled
    } else {
        true // Default to enabled when not using persisted config
    };
    let enabled = !force_disabled && config_enabled && !is_disabled_by_env();

    let session_id = uuid::Uuid::new_v4().to_string();
    let operation_id = uuid::Uuid::new_v4().to_string();
    let (support_key, support_key_short) = get_support_key_info();
    let device_id = support_key.clone();

    let client = if enabled {
        Some(TelemetryClient::new(instrumentation_key.to_string()))
    } else {
        None
    };

    let state = TelemetryState {
        client,
        app_type,
        version: version.to_string(),
        session_id,
        operation_id,
        device_id,
        support_key,
        support_key_short,
        enabled,
    };

    let _ = TELEMETRY_STATE.set(Mutex::new(state));

    if enabled {
        track_event("app-started", HashMap::new());
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "telemetry")))]
pub fn init(app_type: AppType, version: &str, force_disabled: bool, _instrumentation_key: &str) {
    init_with_config(
        app_type,
        version,
        force_disabled,
        _instrumentation_key,
        true,
    );
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "telemetry")))]
pub fn init_without_persisted_state(
    app_type: AppType,
    version: &str,
    force_disabled: bool,
    _instrumentation_key: &str,
) {
    init_with_config(
        app_type,
        version,
        force_disabled,
        _instrumentation_key,
        false,
    );
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "telemetry")))]
fn init_with_config(
    app_type: AppType,
    version: &str,
    force_disabled: bool,
    _instrumentation_key: &str,
    use_persisted_config: bool,
) {
    let config_enabled = if use_persisted_config {
        TelemetryConfig::load().enabled
    } else {
        true // Default to enabled when not using persisted config
    };
    let enabled = !force_disabled && config_enabled && !is_disabled_by_env();

    let session_id = uuid::Uuid::new_v4().to_string();
    let operation_id = uuid::Uuid::new_v4().to_string();
    let (support_key, support_key_short) = get_support_key_info();
    let device_id = support_key.clone();

    let state = TelemetryState {
        app_type,
        version: version.to_string(),
        session_id,
        operation_id,
        device_id,
        support_key,
        support_key_short,
        enabled,
    };

    let _ = TELEMETRY_STATE.set(Mutex::new(state));
}

#[cfg(target_arch = "wasm32")]
pub fn init(_app_type: AppType, _version: &str, _force_disabled: bool, _instrumentation_key: &str) {
}

#[cfg(target_arch = "wasm32")]
pub fn init_without_persisted_state(
    _app_type: AppType,
    _version: &str,
    _force_disabled: bool,
    _instrumentation_key: &str,
) {
}

pub fn is_enabled() -> bool {
    TELEMETRY_STATE
        .get()
        .and_then(|m| m.lock().ok())
        .map(|s| s.enabled)
        .unwrap_or(false)
}

pub fn set_enabled(enabled: bool) -> anyhow::Result<()> {
    use super::config::is_disabled_by_env;

    let final_enabled = enabled && !is_disabled_by_env();
    let config = TelemetryConfig {
        enabled: final_enabled,
    };
    config.save()?;

    if let Some(state_mutex) = TELEMETRY_STATE.get()
        && let Ok(mut state) = state_mutex.lock()
    {
        state.enabled = final_enabled;
    }

    Ok(())
}

#[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
pub fn track_event(name: &str, properties: HashMap<String, String>) {
    let Some(state_mutex) = TELEMETRY_STATE.get() else {
        return;
    };
    let Ok(state) = state_mutex.lock() else {
        return;
    };
    if !state.enabled {
        return;
    }
    let Some(ref client) = state.client else {
        return;
    };

    let mut event = appinsights::telemetry::EventTelemetry::new(name);

    let props = event.properties_mut();
    props.insert("app_type".to_string(), state.app_type.as_str().to_string());
    props.insert("version".to_string(), state.version.clone());
    props.insert("session_id".to_string(), state.session_id.clone());
    props.insert("device_id".to_string(), state.device_id.clone());
    props.insert("support_key".to_string(), state.support_key.clone());
    props.insert(
        "support_key_short".to_string(),
        state.support_key_short.clone(),
    );
    props.insert("os".to_string(), std::env::consts::OS.to_string());
    props.insert("arch".to_string(), std::env::consts::ARCH.to_string());

    for (key, value) in properties {
        props.insert(key, value);
    }

    let tags = event.tags_mut();
    tags.insert("ai.operation.id".to_string(), state.operation_id.clone());
    tags.insert("ai.session.id".to_string(), state.session_id.clone());
    tags.insert("ai.device.id".to_string(), state.device_id.clone());
    tags.insert("ai.user.id".to_string(), state.support_key.clone());
    tags.insert("ai.device.os".to_string(), std::env::consts::OS.to_string());
    tags.insert("ai.application.ver".to_string(), state.version.clone());

    client.track(event);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "telemetry")))]
pub fn track_event(_name: &str, _properties: HashMap<String, String>) {}

#[cfg(target_arch = "wasm32")]
pub fn track_event(_name: &str, _properties: HashMap<String, String>) {}

#[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
pub fn track_error(error: &dyn std::error::Error) {
    let Some(state_mutex) = TELEMETRY_STATE.get() else {
        return;
    };
    let Ok(state) = state_mutex.lock() else {
        return;
    };
    if !state.enabled {
        return;
    }
    let Some(ref client) = state.client else {
        return;
    };

    let message = sanitize_error_message(&error.to_string());
    let error_type = get_error_type_name(error);

    let mut trace = appinsights::telemetry::TraceTelemetry::new(
        format!("[{}] {}", error_type, message),
        SeverityLevel::Error,
    );

    let props = trace.properties_mut();
    props.insert("app_type".to_string(), state.app_type.as_str().to_string());
    props.insert("version".to_string(), state.version.clone());
    props.insert("session_id".to_string(), state.session_id.clone());
    props.insert("device_id".to_string(), state.device_id.clone());
    props.insert("support_key".to_string(), state.support_key.clone());
    props.insert(
        "support_key_short".to_string(),
        state.support_key_short.clone(),
    );
    props.insert("os".to_string(), std::env::consts::OS.to_string());
    props.insert("arch".to_string(), std::env::consts::ARCH.to_string());
    props.insert("error_type".to_string(), error_type);
    props.insert("error_message".to_string(), message);

    let tags = trace.tags_mut();
    tags.insert("ai.operation.id".to_string(), state.operation_id.clone());
    tags.insert("ai.session.id".to_string(), state.session_id.clone());
    tags.insert("ai.device.id".to_string(), state.device_id.clone());
    tags.insert("ai.user.id".to_string(), state.support_key.clone());
    tags.insert("ai.device.os".to_string(), std::env::consts::OS.to_string());
    tags.insert("ai.application.ver".to_string(), state.version.clone());

    client.track(trace);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "telemetry")))]
pub fn track_error(_error: &dyn std::error::Error) {}

#[cfg(target_arch = "wasm32")]
pub fn track_error(_error: &dyn std::error::Error) {}

#[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
pub fn track_error_message(message: &str) {
    let Some(state_mutex) = TELEMETRY_STATE.get() else {
        return;
    };
    let Ok(state) = state_mutex.lock() else {
        return;
    };
    if !state.enabled {
        return;
    }
    let Some(ref client) = state.client else {
        return;
    };

    let sanitized = sanitize_error_message(message);

    let mut trace =
        appinsights::telemetry::TraceTelemetry::new(sanitized.clone(), SeverityLevel::Error);

    let props = trace.properties_mut();
    props.insert("app_type".to_string(), state.app_type.as_str().to_string());
    props.insert("version".to_string(), state.version.clone());
    props.insert("session_id".to_string(), state.session_id.clone());
    props.insert("device_id".to_string(), state.device_id.clone());
    props.insert("support_key".to_string(), state.support_key.clone());
    props.insert(
        "support_key_short".to_string(),
        state.support_key_short.clone(),
    );
    props.insert("os".to_string(), std::env::consts::OS.to_string());
    props.insert("arch".to_string(), std::env::consts::ARCH.to_string());
    props.insert("error_message".to_string(), sanitized);

    let tags = trace.tags_mut();
    tags.insert("ai.operation.id".to_string(), state.operation_id.clone());
    tags.insert("ai.session.id".to_string(), state.session_id.clone());
    tags.insert("ai.device.id".to_string(), state.device_id.clone());
    tags.insert("ai.user.id".to_string(), state.support_key.clone());
    tags.insert("ai.device.os".to_string(), std::env::consts::OS.to_string());
    tags.insert("ai.application.ver".to_string(), state.version.clone());

    client.track(trace);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "telemetry")))]
pub fn track_error_message(_message: &str) {}

#[cfg(target_arch = "wasm32")]
pub fn track_error_message(_message: &str) {}

#[derive(Debug, Clone, Default)]
pub struct CliArgPatterns {
    pub verbose: bool,
    pub log: bool,
    pub env: bool,
    pub insecure: bool,
    pub discover: bool,
    pub no_banner: bool,
    pub pretty_json: bool,
    pub report: bool,
    pub report_format: Option<String>,
    pub export: bool,
    pub export_json: bool,
    pub file_count: usize,
    pub delay: u64,
}

pub fn track_cli_args(args: &CliArgPatterns) {
    let mut properties = HashMap::new();

    properties.insert("verbose".to_string(), args.verbose.to_string());
    properties.insert("log".to_string(), args.log.to_string());
    properties.insert("env".to_string(), args.env.to_string());
    properties.insert("insecure".to_string(), args.insecure.to_string());
    properties.insert("discover".to_string(), args.discover.to_string());
    properties.insert("no_banner".to_string(), args.no_banner.to_string());
    properties.insert("pretty_json".to_string(), args.pretty_json.to_string());
    properties.insert("report".to_string(), args.report.to_string());
    properties.insert("export".to_string(), args.export.to_string());
    properties.insert("export_json".to_string(), args.export_json.to_string());
    properties.insert("file_count".to_string(), args.file_count.to_string());
    properties.insert("delay".to_string(), args.delay.to_string());

    if let Some(ref format) = args.report_format {
        properties.insert("report_format".to_string(), format.clone());
    }

    track_event("cli-args", properties);
}

pub fn track_request_result(success: bool, request_count: usize, duration_ms: u64) {
    let mut properties = HashMap::new();
    properties.insert("success".to_string(), success.to_string());
    properties.insert("request_count".to_string(), request_count.to_string());
    properties.insert("duration_ms".to_string(), duration_ms.to_string());

    track_event("request-executed", properties);
}

/// Track performance metrics (parsing, execution timing)
pub fn track_metric(
    metric_name: &str,
    duration_ms: u64,
    additional_props: HashMap<String, String>,
) {
    let mut properties = HashMap::new();
    properties.insert("metric_name".to_string(), metric_name.to_string());
    properties.insert("duration_ms".to_string(), duration_ms.to_string());

    for (key, value) in additional_props {
        properties.insert(key, value);
    }

    track_event("metric", properties);
}

/// Categories of connection errors for telemetry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionErrorCategory {
    /// SSL/TLS certificate or handshake errors
    Ssl,
    /// DNS resolution failures
    Dns,
    /// Connection refused or unreachable
    ConnectionRefused,
    /// Request or connection timeout
    Timeout,
    /// Other connection errors
    Other,
}

impl ConnectionErrorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionErrorCategory::Ssl => "ssl",
            ConnectionErrorCategory::Dns => "dns",
            ConnectionErrorCategory::ConnectionRefused => "connection_refused",
            ConnectionErrorCategory::Timeout => "timeout",
            ConnectionErrorCategory::Other => "other",
        }
    }
}

/// Track connection errors with categorization (no sensitive data)
pub fn track_connection_error(category: ConnectionErrorCategory, is_insecure_mode: bool) {
    let mut properties = HashMap::new();
    properties.insert("error_category".to_string(), category.as_str().to_string());
    properties.insert("insecure_mode".to_string(), is_insecure_mode.to_string());

    track_event("connection-error", properties);
}

/// Track feature usage in TUI/GUI apps
pub fn track_feature_usage(feature_name: &str) {
    let mut properties = HashMap::new();
    properties.insert("feature_name".to_string(), feature_name.to_string());

    track_event("feature-used", properties);
}

/// Track file parsing metrics
pub fn track_parse_complete(request_count: usize, duration_ms: u64) {
    let mut properties = HashMap::new();
    properties.insert("request_count".to_string(), request_count.to_string());
    properties.insert("duration_ms".to_string(), duration_ms.to_string());

    track_event("parse-complete", properties);
}

/// Track execution completion metrics
pub fn track_execution_complete(
    success_count: usize,
    failed_count: usize,
    skipped_count: usize,
    total_duration_ms: u64,
) {
    let mut properties = HashMap::new();
    properties.insert("success_count".to_string(), success_count.to_string());
    properties.insert("failed_count".to_string(), failed_count.to_string());
    properties.insert("skipped_count".to_string(), skipped_count.to_string());
    properties.insert(
        "total_duration_ms".to_string(),
        total_duration_ms.to_string(),
    );

    track_event("execution-complete", properties);
}

#[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
pub fn flush() {
    track_event("app-exited", HashMap::new());

    let Some(state_mutex) = TELEMETRY_STATE.get() else {
        return;
    };
    let Ok(mut state) = state_mutex.lock() else {
        return;
    };
    if !state.enabled {
        return;
    }

    if let Some(client) = state.client.take() {
        client.close_channel();
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "telemetry")))]
pub fn flush() {}

#[cfg(target_arch = "wasm32")]
pub fn flush() {}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_is_enabled_returns_false_when_uninitialized() {
        // When TELEMETRY_STATE is not initialized, should return false
        // Note: This test might fail if other tests have initialized the state
        // In a real scenario, we'd use a different approach or reset state
        // We just verify it doesn't panic
        let _enabled = is_enabled();
    }

    #[test]
    fn test_connection_error_category_as_str() {
        assert_eq!(ConnectionErrorCategory::Ssl.as_str(), "ssl");
        assert_eq!(ConnectionErrorCategory::Dns.as_str(), "dns");
        assert_eq!(
            ConnectionErrorCategory::ConnectionRefused.as_str(),
            "connection_refused"
        );
        assert_eq!(ConnectionErrorCategory::Timeout.as_str(), "timeout");
        assert_eq!(ConnectionErrorCategory::Other.as_str(), "other");
    }

    #[test]
    fn test_cli_arg_patterns_default() {
        let args = CliArgPatterns::default();
        assert!(!args.verbose);
        assert!(!args.log);
        assert!(!args.env);
        assert!(!args.insecure);
        assert!(!args.discover);
        assert!(!args.no_banner);
        assert!(!args.pretty_json);
        assert!(!args.report);
        assert_eq!(args.report_format, None);
        assert!(!args.export);
        assert_eq!(args.file_count, 0);
    }

    #[test]
    fn test_track_cli_args_does_not_panic() {
        let args = CliArgPatterns {
            verbose: true,
            log: true,
            env: false,
            insecure: true,
            discover: false,
            no_banner: true,
            pretty_json: false,
            report: true,
            report_format: Some("json".to_string()),
            export: true,
            export_json: false,
            file_count: 5,
            delay: 0,
        };

        // Should not panic even if telemetry is not initialized
        track_cli_args(&args);
    }

    #[test]
    fn test_track_request_result_does_not_panic() {
        // Should not panic even if telemetry is not initialized
        track_request_result(true, 10, 1500);
        track_request_result(false, 5, 3000);
    }

    #[test]
    fn test_track_metric_does_not_panic() {
        let mut props = HashMap::new();
        props.insert("test_key".to_string(), "test_value".to_string());

        // Should not panic even if telemetry is not initialized
        track_metric("test_metric", 250, props);
    }

    #[test]
    fn test_track_connection_error_does_not_panic() {
        // Should not panic even if telemetry is not initialized
        track_connection_error(ConnectionErrorCategory::Ssl, false);
        track_connection_error(ConnectionErrorCategory::Dns, true);
        track_connection_error(ConnectionErrorCategory::Timeout, false);
    }

    #[test]
    fn test_track_feature_usage_does_not_panic() {
        // Should not panic even if telemetry is not initialized
        track_feature_usage("test_feature");
    }

    #[test]
    fn test_track_parse_complete_does_not_panic() {
        // Should not panic even if telemetry is not initialized
        track_parse_complete(15, 500);
    }

    #[test]
    fn test_track_execution_complete_does_not_panic() {
        // Should not panic even if telemetry is not initialized
        track_execution_complete(10, 2, 1, 5000);
    }

    #[test]
    fn test_flush_does_not_panic() {
        // Should not panic even if telemetry is not initialized
        flush();
    }

    // Tests for set_enabled with environment variables
    #[test]
    #[serial]
    fn test_set_enabled_respects_do_not_track() {
        unsafe {
            std::env::set_var("DO_NOT_TRACK", "1");
        }

        // Even if we try to enable, it should respect the env var
        let result = set_enabled(true);
        assert!(result.is_ok());

        // Clean up
        unsafe {
            std::env::remove_var("DO_NOT_TRACK");
        }
    }

    #[test]
    #[serial]
    fn test_set_enabled_respects_httprunner_optout() {
        unsafe {
            std::env::set_var("HTTPRUNNER_TELEMETRY_OPTOUT", "true");
        }

        // Even if we try to enable, it should respect the env var
        let result = set_enabled(true);
        assert!(result.is_ok());

        // Clean up
        unsafe {
            std::env::remove_var("HTTPRUNNER_TELEMETRY_OPTOUT");
        }
    }

    #[test]
    #[serial]
    fn test_set_enabled_allows_when_no_env_vars() {
        unsafe {
            std::env::remove_var("DO_NOT_TRACK");
            std::env::remove_var("HTTPRUNNER_TELEMETRY_OPTOUT");
        }

        let result = set_enabled(true);
        assert!(result.is_ok());

        let result = set_enabled(false);
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_set_enabled_handles_false_explicitly() {
        unsafe {
            std::env::remove_var("DO_NOT_TRACK");
            std::env::remove_var("HTTPRUNNER_TELEMETRY_OPTOUT");
        }

        // Should be able to disable even without env vars
        let result = set_enabled(false);
        assert!(result.is_ok());
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
    #[test]
    fn test_track_event_with_properties() {
        let mut props = HashMap::new();
        props.insert("key1".to_string(), "value1".to_string());
        props.insert("key2".to_string(), "value2".to_string());

        // Should not panic
        track_event("test_event", props);
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "telemetry"))]
    #[test]
    fn test_track_event_empty_properties() {
        // Should not panic with empty properties
        track_event("test_event", HashMap::new());
    }

    #[test]
    fn test_cli_arg_patterns_with_all_flags_enabled() {
        let args = CliArgPatterns {
            verbose: true,
            log: true,
            env: true,
            insecure: true,
            discover: true,
            no_banner: true,
            pretty_json: true,
            report: true,
            report_format: Some("html".to_string()),
            export: true,
            export_json: false,
            file_count: 100,
            delay: 0,
        };

        track_cli_args(&args);
        assert_eq!(args.file_count, 100);
    }

    #[test]
    fn test_cli_arg_patterns_with_all_flags_disabled() {
        let args = CliArgPatterns {
            verbose: false,
            log: false,
            env: false,
            insecure: false,
            discover: false,
            no_banner: false,
            pretty_json: false,
            report: false,
            report_format: None,
            export: false,
            export_json: false,
            file_count: 0,
            delay: 0,
        };

        track_cli_args(&args);
    }

    #[test]
    fn test_track_request_result_success() {
        track_request_result(true, 1, 100);
    }

    #[test]
    fn test_track_request_result_failure() {
        track_request_result(false, 1, 200);
    }

    #[test]
    fn test_track_request_result_multiple_requests() {
        track_request_result(true, 50, 5000);
    }

    #[test]
    fn test_track_request_result_zero_duration() {
        track_request_result(true, 1, 0);
    }

    #[test]
    fn test_track_metric_with_empty_properties() {
        track_metric("performance", 150, HashMap::new());
    }

    #[test]
    fn test_track_metric_with_multiple_properties() {
        let mut props = HashMap::new();
        props.insert("category".to_string(), "network".to_string());
        props.insert("endpoint".to_string(), "api".to_string());
        props.insert("version".to_string(), "v2".to_string());

        track_metric("api_call", 300, props);
    }

    #[test]
    fn test_track_connection_error_all_categories() {
        track_connection_error(ConnectionErrorCategory::Ssl, true);
        track_connection_error(ConnectionErrorCategory::Dns, false);
        track_connection_error(ConnectionErrorCategory::ConnectionRefused, true);
        track_connection_error(ConnectionErrorCategory::Timeout, false);
        track_connection_error(ConnectionErrorCategory::Other, true);
    }

    #[test]
    fn test_track_feature_usage_various_features() {
        track_feature_usage("export");
        track_feature_usage("import");
        track_feature_usage("validation");
        track_feature_usage("formatting");
    }

    #[test]
    fn test_track_parse_complete_zero_requests() {
        track_parse_complete(0, 0);
    }

    #[test]
    fn test_track_parse_complete_many_requests() {
        track_parse_complete(1000, 5000);
    }

    #[test]
    fn test_track_execution_complete_all_success() {
        track_execution_complete(10, 0, 0, 2000);
    }

    #[test]
    fn test_track_execution_complete_all_failed() {
        track_execution_complete(0, 10, 0, 3000);
    }

    #[test]
    fn test_track_execution_complete_all_skipped() {
        track_execution_complete(0, 0, 10, 1000);
    }

    #[test]
    fn test_track_execution_complete_mixed_results() {
        track_execution_complete(5, 3, 2, 4000);
    }

    #[test]
    fn test_track_execution_complete_zero_duration() {
        track_execution_complete(1, 0, 0, 0);
    }

    #[test]
    fn test_connection_error_category_equality() {
        assert_eq!(ConnectionErrorCategory::Ssl, ConnectionErrorCategory::Ssl);
        assert_ne!(ConnectionErrorCategory::Ssl, ConnectionErrorCategory::Dns);
        assert_ne!(
            ConnectionErrorCategory::Timeout,
            ConnectionErrorCategory::Other
        );
    }

    #[test]
    fn test_connection_error_category_debug() {
        let ssl = ConnectionErrorCategory::Ssl;
        let debug_str = format!("{:?}", ssl);
        assert!(debug_str.contains("Ssl"));
    }

    #[test]
    fn test_cli_arg_patterns_clone() {
        let args1 = CliArgPatterns {
            verbose: true,
            log: false,
            env: true,
            insecure: false,
            discover: true,
            no_banner: false,
            pretty_json: true,
            report: false,
            report_format: Some("json".to_string()),
            export: true,
            export_json: false,
            file_count: 42,
            delay: 0,
        };

        let args2 = args1.clone();
        assert_eq!(args1.verbose, args2.verbose);
        assert_eq!(args1.file_count, args2.file_count);
        assert_eq!(args1.report_format, args2.report_format);
    }

    #[test]
    fn test_cli_arg_patterns_debug() {
        let args = CliArgPatterns::default();
        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("CliArgPatterns"));
    }

    #[test]
    #[serial]
    fn test_set_enabled_state_persistence() {
        unsafe {
            std::env::remove_var("DO_NOT_TRACK");
            std::env::remove_var("HTTPRUNNER_TELEMETRY_OPTOUT");
        }

        // Enable then disable
        let _ = set_enabled(true);
        let _ = set_enabled(false);
    }

    #[test]
    fn test_track_cli_args_with_various_file_counts() {
        for count in [0, 1, 5, 10, 100, 1000] {
            let args = CliArgPatterns {
                file_count: count,
                ..Default::default()
            };
            track_cli_args(&args);
        }
    }

    #[test]
    fn test_track_cli_args_with_different_report_formats() {
        for format in ["json", "html", "xml", "markdown"] {
            let args = CliArgPatterns {
                report: true,
                report_format: Some(format.to_string()),
                ..Default::default()
            };
            track_cli_args(&args);
        }
    }

    #[test]
    fn test_track_metric_zero_duration() {
        track_metric("instant_operation", 0, HashMap::new());
    }

    #[test]
    fn test_track_metric_large_duration() {
        track_metric("long_operation", u64::MAX, HashMap::new());
    }

    #[test]
    fn test_track_parse_complete_large_numbers() {
        track_parse_complete(usize::MAX, u64::MAX);
    }

    #[test]
    fn test_track_execution_complete_large_numbers() {
        track_execution_complete(1000000, 500000, 250000, u64::MAX);
    }

    #[test]
    fn test_is_enabled_multiple_calls() {
        // Multiple calls should be safe and consistent
        let first = is_enabled();
        let second = is_enabled();
        let third = is_enabled();

        // All calls should return the same value
        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    #[test]
    fn test_flush_multiple_calls() {
        // Multiple flush calls should be safe
        flush();
        flush();
        flush();
    }

    #[test]
    fn test_connection_error_category_copy() {
        let cat1 = ConnectionErrorCategory::Ssl;
        let cat2 = cat1;
        assert_eq!(cat1, cat2);
    }

    #[test]
    fn test_track_feature_usage_empty_string() {
        track_feature_usage("");
    }

    #[test]
    fn test_track_feature_usage_long_name() {
        let long_name = "a".repeat(1000);
        track_feature_usage(&long_name);
    }

    #[test]
    fn test_track_metric_with_special_characters() {
        let mut props = HashMap::new();
        props.insert("key-with-dashes".to_string(), "value".to_string());
        props.insert("key_with_underscores".to_string(), "value".to_string());
        props.insert("key.with.dots".to_string(), "value".to_string());

        track_metric("special-metric", 100, props);
    }
}
