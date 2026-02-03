use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
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
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config path"))?;

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

pub fn is_disabled_by_env() -> bool {
    if env::var("DO_NOT_TRACK").is_ok_and(|v| v == "1" || v.to_lowercase() == "true") {
        return true;
    }
    if env::var("HTTPRUNNER_TELEMETRY_OPTOUT").is_ok_and(|v| v == "1" || v.to_lowercase() == "true")
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(config.enabled);
    }

    #[test]
    #[serial]
    fn test_is_disabled_by_env_do_not_track_1() {
        unsafe {
            std::env::set_var("DO_NOT_TRACK", "1");
        }
        assert!(is_disabled_by_env());
        unsafe {
            std::env::remove_var("DO_NOT_TRACK");
        }
    }

    #[test]
    #[serial]
    fn test_is_disabled_by_env_do_not_track_true() {
        unsafe {
            std::env::set_var("DO_NOT_TRACK", "true");
        }
        assert!(is_disabled_by_env());
        unsafe {
            std::env::remove_var("DO_NOT_TRACK");
        }
    }

    #[test]
    #[serial]
    fn test_is_disabled_by_env_do_not_track_true_uppercase() {
        unsafe {
            std::env::set_var("DO_NOT_TRACK", "TRUE");
        }
        assert!(is_disabled_by_env());
        unsafe {
            std::env::remove_var("DO_NOT_TRACK");
        }
    }

    #[test]
    #[serial]
    fn test_is_disabled_by_env_httprunner_optout_1() {
        unsafe {
            std::env::set_var("HTTPRUNNER_TELEMETRY_OPTOUT", "1");
        }
        assert!(is_disabled_by_env());
        unsafe {
            std::env::remove_var("HTTPRUNNER_TELEMETRY_OPTOUT");
        }
    }

    #[test]
    #[serial]
    fn test_is_disabled_by_env_httprunner_optout_true() {
        unsafe {
            std::env::set_var("HTTPRUNNER_TELEMETRY_OPTOUT", "true");
        }
        assert!(is_disabled_by_env());
        unsafe {
            std::env::remove_var("HTTPRUNNER_TELEMETRY_OPTOUT");
        }
    }

    #[test]
    #[serial]
    fn test_is_disabled_by_env_httprunner_optout_true_mixed_case() {
        unsafe {
            std::env::set_var("HTTPRUNNER_TELEMETRY_OPTOUT", "TrUe");
        }
        assert!(is_disabled_by_env());
        unsafe {
            std::env::remove_var("HTTPRUNNER_TELEMETRY_OPTOUT");
        }
    }

    #[test]
    #[serial]
    fn test_is_disabled_by_env_not_set() {
        unsafe {
            std::env::remove_var("DO_NOT_TRACK");
            std::env::remove_var("HTTPRUNNER_TELEMETRY_OPTOUT");
        }
        assert!(!is_disabled_by_env());
    }

    #[test]
    #[serial]
    fn test_is_disabled_by_env_invalid_values() {
        unsafe {
            std::env::set_var("DO_NOT_TRACK", "0");
            std::env::set_var("HTTPRUNNER_TELEMETRY_OPTOUT", "false");
        }
        assert!(!is_disabled_by_env());
        unsafe {
            std::env::remove_var("DO_NOT_TRACK");
            std::env::remove_var("HTTPRUNNER_TELEMETRY_OPTOUT");
        }
    }

    #[test]
    #[serial]
    fn test_is_disabled_by_env_empty_string() {
        unsafe {
            std::env::set_var("DO_NOT_TRACK", "");
            std::env::set_var("HTTPRUNNER_TELEMETRY_OPTOUT", "");
        }
        assert!(!is_disabled_by_env());
        unsafe {
            std::env::remove_var("DO_NOT_TRACK");
            std::env::remove_var("HTTPRUNNER_TELEMETRY_OPTOUT");
        }
    }

    #[test]
    fn test_telemetry_config_serialization() {
        let config = TelemetryConfig { enabled: true };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enabled\":true"));

        let deserialized: TelemetryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.enabled, deserialized.enabled);
    }

    #[test]
    fn test_telemetry_config_deserialization_missing_field() {
        // Test that missing 'enabled' field defaults to true
        let json = "{}";
        let config: TelemetryConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
    }
}
