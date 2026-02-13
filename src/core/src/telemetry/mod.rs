//! Telemetry module for Azure Application Insights integration.
//!
//! Privacy: No HTTP content, file paths, URLs, or env values are collected.
//!
//! Opt-out: Set `HTTPRUNNER_TELEMETRY_OPTOUT=1`, `DO_NOT_TRACK=1`, or `--no-telemetry` (CLI only).
//!
//! ## CLI vs GUI/TUI Behavior
//!
//! - **CLI**: Uses `init_without_persisted_state()` - respects only `--no-telemetry` flag and environment variables
//! - **GUI/TUI**: Uses `init()` - respects persisted user preferences, environment variables, and UI toggles

mod app_type;
mod config;
mod sanitize;
mod tracking;

pub use app_type::AppType;
pub use config::TelemetryConfig;
pub use tracking::{
    CliArgPatterns, ConnectionErrorCategory, flush, init, init_without_persisted_state, is_enabled,
    set_enabled, track_cli_args, track_connection_error, track_error, track_error_message,
    track_event, track_execution_complete, track_feature_usage, track_metric, track_parse_complete,
    track_request_result,
};
