//! Telemetry module for Azure Application Insights integration.
//!
//! This module provides telemetry collection for httprunner while respecting user privacy.
//!
//! ## Privacy Guarantees
//! - NO HTTP request/response content is ever collected
//! - NO file paths or URL values are collected
//! - NO environment variable values are collected
//! - Only anonymized usage patterns and error types are tracked
//!
//! ## Opt-Out Options
//! - Set `HTTPRUNNER_TELEMETRY_OPTOUT=1` environment variable
//! - Set `DO_NOT_TRACK=1` environment variable (standard opt-out)
//! - Use `--no-telemetry` CLI flag
//! - Disable in TUI/GUI settings

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use appinsights::blocking::TelemetryClient;
#[cfg(not(target_arch = "wasm32"))]
use appinsights::telemetry::{SeverityLevel, Telemetry};

// Azure Application Insights instrumentation key
// This is a write-only ingestion key - it cannot be used to read data
const INSTRUMENTATION_KEY: &str = "a7a07a35-4869-4fa2-b852-03f44b35f418";

static TELEMETRY_STATE: OnceLock<Mutex<TelemetryState>> = OnceLock::new();

/// Application type for distinguishing telemetry sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppType {
    Cli,
    Tui,
    Gui,
}

impl AppType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppType::Cli => "CLI",
            AppType::Tui => "TUI",
            AppType::Gui => "GUI",
        }
    }
}

impl std::fmt::Display for AppType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Telemetry configuration persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl TelemetryConfig {
    /// Load telemetry config from disk
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Save telemetry config to disk
    pub fn save(&self) -> anyhow::Result<()> {
        let path =
            Self::config_path().ok_or_else(|| anyhow::anyhow!("Could not determine config path"))?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("httprunner").join("telemetry.json"))
    }
}

/// Internal telemetry state
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

/// Check if telemetry is disabled via environment variables
fn is_telemetry_disabled_by_env() -> bool {
    // Check standard DO_NOT_TRACK
    if env::var("DO_NOT_TRACK").is_ok_and(|v| v == "1" || v.to_lowercase() == "true") {
        return true;
    }

    // Check httprunner-specific opt-out
    if env::var("HTTPRUNNER_TELEMETRY_OPTOUT").is_ok_and(|v| v == "1" || v.to_lowercase() == "true")
    {
        return true;
    }

    false
}

/// Get support key information
fn get_support_key_info() -> (String, String) {
    if let Ok(support_key) = crate::logging::get_support_key() {
        (support_key.key, support_key.short_key)
    } else {
        let key = uuid::Uuid::new_v4().to_string();
        let short_key = key.chars().take(8).collect();
        (key, short_key)
    }
}

/// Initialize telemetry for the application.
///
/// This should be called once at application startup.
///
/// # Arguments
/// * `app_type` - The type of application (CLI, TUI, or GUI)
/// * `version` - The application version string
/// * `force_disabled` - If true, telemetry will be disabled regardless of other settings
#[cfg(not(target_arch = "wasm32"))]
pub fn init(app_type: AppType, version: &str, force_disabled: bool) {
    let config = TelemetryConfig::load();
    let enabled = !force_disabled
        && config.enabled
        && !is_telemetry_disabled_by_env()
        && INSTRUMENTATION_KEY != "YOUR_INSTRUMENTATION_KEY_HERE";

    let session_id = uuid::Uuid::new_v4().to_string();
    let (support_key, support_key_short) = get_support_key_info();
    let device_id = support_key.clone();

    // Create the App Insights client
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

    // Track app start event
    if enabled {
        track_event("AppStart", HashMap::new());
    }
}

/// Initialize telemetry (WASM stub - no-op)
#[cfg(target_arch = "wasm32")]
pub fn init(_app_type: AppType, _version: &str, _force_disabled: bool) {
    // Telemetry not supported on WASM
}

/// Check if telemetry is currently enabled
pub fn is_enabled() -> bool {
    TELEMETRY_STATE
        .get()
        .and_then(|m| m.lock().ok())
        .map(|s| s.enabled)
        .unwrap_or(false)
}

/// Set telemetry enabled state and persist to config
pub fn set_enabled(enabled: bool) -> anyhow::Result<()> {
    let config = TelemetryConfig { enabled };
    config.save()
}

/// Track a custom event with properties
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

    // Create event telemetry
    let mut event = appinsights::telemetry::EventTelemetry::new(name);
    
    // Add standard properties
    let props = event.properties_mut();
    props.insert("app_type".to_string(), state.app_type.as_str().to_string());
    props.insert("version".to_string(), state.version.clone());
    props.insert("session_id".to_string(), state.session_id.clone());
    props.insert("device_id".to_string(), state.device_id.clone());
    props.insert("support_key".to_string(), state.support_key.clone());
    props.insert("support_key_short".to_string(), state.support_key_short.clone());
    props.insert("os".to_string(), std::env::consts::OS.to_string());
    props.insert("arch".to_string(), std::env::consts::ARCH.to_string());
    
    // Add custom properties
    for (key, value) in properties {
        props.insert(key, value);
    }
    
    // Set context tags
    let tags = event.tags_mut();
    tags.insert("ai.session.id".to_string(), state.session_id.clone());
    tags.insert("ai.device.id".to_string(), state.device_id.clone());
    tags.insert("ai.user.id".to_string(), state.support_key.clone());
    tags.insert("ai.device.os".to_string(), std::env::consts::OS.to_string());
    tags.insert("ai.application.ver".to_string(), state.version.clone());

    client.track(event);
}

/// Track a custom event (WASM stub - no-op)
#[cfg(target_arch = "wasm32")]
pub fn track_event(_name: &str, _properties: HashMap<String, String>) {
    // Telemetry not supported on WASM
}

/// Track an error/exception
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

    // Sanitize error message - remove potential file paths and sensitive data
    let message = sanitize_error_message(&error.to_string());
    let error_type = get_error_type_name(error);

    // Create trace telemetry for the error
    let mut trace = appinsights::telemetry::TraceTelemetry::new(
        format!("[{}] {}", error_type, message),
        SeverityLevel::Error,
    );
    
    // Add properties
    let props = trace.properties_mut();
    props.insert("app_type".to_string(), state.app_type.as_str().to_string());
    props.insert("version".to_string(), state.version.clone());
    props.insert("session_id".to_string(), state.session_id.clone());
    props.insert("device_id".to_string(), state.device_id.clone());
    props.insert("support_key".to_string(), state.support_key.clone());
    props.insert("support_key_short".to_string(), state.support_key_short.clone());
    props.insert("os".to_string(), std::env::consts::OS.to_string());
    props.insert("arch".to_string(), std::env::consts::ARCH.to_string());
    props.insert("error_type".to_string(), error_type);
    props.insert("error_message".to_string(), message);
    
    // Set context tags
    let tags = trace.tags_mut();
    tags.insert("ai.session.id".to_string(), state.session_id.clone());
    tags.insert("ai.device.id".to_string(), state.device_id.clone());
    tags.insert("ai.user.id".to_string(), state.support_key.clone());
    tags.insert("ai.device.os".to_string(), std::env::consts::OS.to_string());
    tags.insert("ai.application.ver".to_string(), state.version.clone());

    client.track(trace);
}

/// Track an error (WASM stub - no-op)
#[cfg(target_arch = "wasm32")]
pub fn track_error(_error: &dyn std::error::Error) {
    // Telemetry not supported on WASM
}

/// Track an error message string
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

    // Create trace telemetry for the error
    let mut trace = appinsights::telemetry::TraceTelemetry::new(
        sanitized.clone(),
        SeverityLevel::Error,
    );
    
    // Add properties
    let props = trace.properties_mut();
    props.insert("app_type".to_string(), state.app_type.as_str().to_string());
    props.insert("version".to_string(), state.version.clone());
    props.insert("session_id".to_string(), state.session_id.clone());
    props.insert("device_id".to_string(), state.device_id.clone());
    props.insert("support_key".to_string(), state.support_key.clone());
    props.insert("support_key_short".to_string(), state.support_key_short.clone());
    props.insert("os".to_string(), std::env::consts::OS.to_string());
    props.insert("arch".to_string(), std::env::consts::ARCH.to_string());
    props.insert("error_message".to_string(), sanitized);
    
    // Set context tags
    let tags = trace.tags_mut();
    tags.insert("ai.session.id".to_string(), state.session_id.clone());
    tags.insert("ai.device.id".to_string(), state.device_id.clone());
    tags.insert("ai.user.id".to_string(), state.support_key.clone());
    tags.insert("ai.device.os".to_string(), std::env::consts::OS.to_string());
    tags.insert("ai.application.ver".to_string(), state.version.clone());

    client.track(trace);
}

/// Track an error message (WASM stub - no-op)
#[cfg(target_arch = "wasm32")]
pub fn track_error_message(_message: &str) {
    // Telemetry not supported on WASM
}

/// Track CLI argument patterns (only flags, not values)
///
/// This function deliberately only tracks which flags were used,
/// NOT their values (to avoid capturing file paths, URLs, etc.)
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

/// CLI argument patterns for telemetry (only tracks flag usage, not values)
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

/// Track request execution result (success/failure only, no content)
pub fn track_request_result(success: bool, request_count: usize, duration_ms: u64) {
    let mut properties = HashMap::new();
    properties.insert("success".to_string(), success.to_string());
    properties.insert("request_count".to_string(), request_count.to_string());
    properties.insert("duration_ms".to_string(), duration_ms.to_string());

    track_event("RequestExecution", properties);
}

/// Flush all pending telemetry data
///
/// This should be called before application exit to ensure all telemetry is sent.
#[cfg(not(target_arch = "wasm32"))]
pub fn flush() {
    // Track app exit event first
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

    // Take ownership of client and close the channel - this will flush and block until done
    if let Some(client) = state.client.take() {
        client.close_channel();
    }
}

/// Flush telemetry (WASM stub - no-op)
#[cfg(target_arch = "wasm32")]
pub fn flush() {
    // Telemetry not supported on WASM
}

/// Sanitize error messages to remove potential sensitive data
fn sanitize_error_message(message: &str) -> String {
    let mut sanitized = message.to_string();

    // Remove potential URLs first (before path matching, as URLs contain paths)
    if let Ok(url_re) = regex::Regex::new(r"https?://[^\s]+") {
        sanitized = url_re.replace_all(&sanitized, "[URL]").to_string();
    }

    // Remove potential file paths (Unix and Windows)
    let path_patterns = [
        r#"[A-Za-z]:\\[^\s:*?"<>|]+"#, // Windows paths
        r"~[/\\][^\s]*",               // Home directory paths
        r"(/[a-zA-Z0-9_\-./]+)+",      // Unix paths (last to avoid matching URL remnants)
    ];

    for pattern in path_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            sanitized = re.replace_all(&sanitized, "[PATH]").to_string();
        }
    }

    // Remove potential IP addresses
    if let Ok(ip_re) = regex::Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b") {
        sanitized = ip_re.replace_all(&sanitized, "[IP]").to_string();
    }

    // Remove potential email addresses
    if let Ok(email_re) = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
    {
        sanitized = email_re.replace_all(&sanitized, "[EMAIL]").to_string();
    }

    // Truncate very long messages
    if sanitized.len() > 500 {
        sanitized.truncate(500);
        sanitized.push_str("...[TRUNCATED]");
    }

    sanitized
}

/// Get the type name of an error for categorization
#[cfg(not(target_arch = "wasm32"))]
fn get_error_type_name(error: &dyn std::error::Error) -> String {
    // Try to get a meaningful error type name
    let full_type = std::any::type_name_of_val(error);

    // Extract just the type name without the full path
    full_type
        .rsplit("::")
        .next()
        .unwrap_or("Unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_error_message_removes_unix_paths() {
        let msg = "Failed to open /home/user/secret/file.txt";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("/home/user"));
        assert!(sanitized.contains("[PATH]"));
    }

    #[test]
    fn test_sanitize_error_message_removes_windows_paths() {
        let msg = "Failed to open C:\\Users\\Admin\\Documents\\secret.txt";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("C:\\Users"));
        assert!(sanitized.contains("[PATH]"));
    }

    #[test]
    fn test_sanitize_error_message_removes_urls() {
        let msg = "Failed to connect to https://api.secret-server.com/v1/data";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("https://"));
        assert!(sanitized.contains("[URL]"));
    }

    #[test]
    fn test_sanitize_error_message_removes_ip_addresses() {
        let msg = "Connection refused to 192.168.1.100:8080";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("192.168.1.100"));
        assert!(sanitized.contains("[IP]"));
    }

    #[test]
    fn test_sanitize_error_message_removes_emails() {
        let msg = "Invalid auth for user@example.com";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("user@example.com"));
        assert!(sanitized.contains("[EMAIL]"));
    }

    #[test]
    fn test_sanitize_error_message_truncates_long_messages() {
        let msg = "x".repeat(1000);
        let sanitized = sanitize_error_message(&msg);
        assert!(sanitized.len() < 600);
        assert!(sanitized.ends_with("[TRUNCATED]"));
    }

    #[test]
    fn test_app_type_display() {
        assert_eq!(AppType::Cli.as_str(), "CLI");
        assert_eq!(AppType::Tui.as_str(), "TUI");
        assert_eq!(AppType::Gui.as_str(), "GUI");
    }

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(config.enabled);
    }
}
