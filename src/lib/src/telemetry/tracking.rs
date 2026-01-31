use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[cfg(not(target_arch = "wasm32"))]
use appinsights::blocking::TelemetryClient;
#[cfg(not(target_arch = "wasm32"))]
use appinsights::telemetry::{SeverityLevel, Telemetry};

use super::app_type::AppType;
use super::config::{TelemetryConfig, is_disabled_by_env};
use super::sanitize::{get_error_type_name, sanitize_error_message};

const INSTRUMENTATION_KEY: &str = "a7a07a35-4869-4fa2-b852-03f44b35f418";

static TELEMETRY_STATE: OnceLock<Mutex<TelemetryState>> = OnceLock::new();

struct TelemetryState {
    #[cfg(not(target_arch = "wasm32"))]
    client: Option<TelemetryClient>,
    app_type: AppType,
    version: String,
    session_id: String,
    device_id: String,
    support_key: String,
    support_key_short: String,
    enabled: bool,
}

fn get_support_key_info() -> (String, String) {
    if let Ok(support_key) = crate::logging::get_support_key() {
        (support_key.key, support_key.short_key)
    } else {
        let key = uuid::Uuid::new_v4().to_string();
        let short_key = key.chars().take(8).collect();
        (key, short_key)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init(app_type: AppType, version: &str, force_disabled: bool) {
    let config = TelemetryConfig::load();
    let enabled = !force_disabled && config.enabled && !is_disabled_by_env();

    let session_id = uuid::Uuid::new_v4().to_string();
    let (support_key, support_key_short) = get_support_key_info();
    let device_id = support_key.clone();

    let client = TelemetryClient::new(INSTRUMENTATION_KEY.to_string());

    let state = TelemetryState {
        client: Some(client),
        app_type,
        version: version.to_string(),
        session_id,
        device_id,
        support_key,
        support_key_short,
        enabled,
    };

    let _ = TELEMETRY_STATE.set(Mutex::new(state));

    if enabled {
        track_event("AppStart", HashMap::new());
    }
}

#[cfg(target_arch = "wasm32")]
pub fn init(_app_type: AppType, _version: &str, _force_disabled: bool) {}

pub fn is_enabled() -> bool {
    TELEMETRY_STATE
        .get()
        .and_then(|m| m.lock().ok())
        .map(|s| s.enabled)
        .unwrap_or(false)
}

pub fn set_enabled(enabled: bool) -> anyhow::Result<()> {
    let config = TelemetryConfig { enabled };
    config.save()
}

#[cfg(not(target_arch = "wasm32"))]
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
    tags.insert("ai.session.id".to_string(), state.session_id.clone());
    tags.insert("ai.device.id".to_string(), state.device_id.clone());
    tags.insert("ai.user.id".to_string(), state.support_key.clone());
    tags.insert("ai.device.os".to_string(), std::env::consts::OS.to_string());
    tags.insert("ai.application.ver".to_string(), state.version.clone());

    client.track(event);
}

#[cfg(target_arch = "wasm32")]
pub fn track_event(_name: &str, _properties: HashMap<String, String>) {}

#[cfg(not(target_arch = "wasm32"))]
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
    tags.insert("ai.session.id".to_string(), state.session_id.clone());
    tags.insert("ai.device.id".to_string(), state.device_id.clone());
    tags.insert("ai.user.id".to_string(), state.support_key.clone());
    tags.insert("ai.device.os".to_string(), std::env::consts::OS.to_string());
    tags.insert("ai.application.ver".to_string(), state.version.clone());

    client.track(trace);
}

#[cfg(target_arch = "wasm32")]
pub fn track_error(_error: &dyn std::error::Error) {}

#[cfg(not(target_arch = "wasm32"))]
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
    tags.insert("ai.session.id".to_string(), state.session_id.clone());
    tags.insert("ai.device.id".to_string(), state.device_id.clone());
    tags.insert("ai.user.id".to_string(), state.support_key.clone());
    tags.insert("ai.device.os".to_string(), std::env::consts::OS.to_string());
    tags.insert("ai.application.ver".to_string(), state.version.clone());

    client.track(trace);
}

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
    pub file_count: usize,
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
    properties.insert("file_count".to_string(), args.file_count.to_string());

    if let Some(ref format) = args.report_format {
        properties.insert("report_format".to_string(), format.clone());
    }

    track_event("CliInvocation", properties);
}

pub fn track_request_result(success: bool, request_count: usize, duration_ms: u64) {
    let mut properties = HashMap::new();
    properties.insert("success".to_string(), success.to_string());
    properties.insert("request_count".to_string(), request_count.to_string());
    properties.insert("duration_ms".to_string(), duration_ms.to_string());

    track_event("RequestExecution", properties);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn flush() {
    track_event("AppExit", HashMap::new());

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

#[cfg(target_arch = "wasm32")]
pub fn flush() {}
