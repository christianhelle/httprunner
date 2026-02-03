use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppType {
    CLI,
    TUI,
    GUI,
}

impl AppType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppType::CLI => "CLI",
            AppType::TUI => "TUI",
            AppType::GUI => "GUI",
        }
    }
}

impl std::fmt::Display for AppType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_type_display() {
        assert_eq!(AppType::CLI.as_str(), "CLI");
        assert_eq!(AppType::TUI.as_str(), "TUI");
        assert_eq!(AppType::GUI.as_str(), "GUI");
    }

    #[test]
    fn test_app_type_equality() {
        assert_eq!(AppType::CLI, AppType::CLI);
        assert_eq!(AppType::TUI, AppType::TUI);
        assert_eq!(AppType::GUI, AppType::GUI);

        assert_ne!(AppType::CLI, AppType::TUI);
        assert_ne!(AppType::CLI, AppType::GUI);
        assert_ne!(AppType::TUI, AppType::GUI);
    }

    #[test]
    fn test_app_type_clone() {
        let cli1 = AppType::CLI;
        let cli2 = cli1.clone();
        assert_eq!(cli1, cli2);
    }

    #[test]
    fn test_app_type_copy() {
        let cli1 = AppType::CLI;
        let cli2 = cli1;
        assert_eq!(cli1, cli2);
    }

    #[test]
    fn test_app_type_debug() {
        let cli = AppType::CLI;
        let debug_str = format!("{:?}", cli);
        assert!(debug_str.contains("CLI"));
    }

    #[test]
    fn test_app_type_display_trait() {
        assert_eq!(format!("{}", AppType::CLI), "CLI");
        assert_eq!(format!("{}", AppType::TUI), "TUI");
        assert_eq!(format!("{}", AppType::GUI), "GUI");
    }

    #[test]
    fn test_app_type_serialization() {
        let cli = AppType::CLI;
        let json = serde_json::to_string(&cli).unwrap();
        assert!(json.contains("CLI"));

        let deserialized: AppType = serde_json::from_str(&json).unwrap();
        assert_eq!(cli, deserialized);
    }

    #[test]
    fn test_app_type_deserialization_cli() {
        let json = r#""CLI""#;
        let app_type: AppType = serde_json::from_str(json).unwrap();
        assert_eq!(app_type, AppType::CLI);
    }

    #[test]
    fn test_app_type_deserialization_tui() {
        let json = r#""TUI""#;
        let app_type: AppType = serde_json::from_str(json).unwrap();
        assert_eq!(app_type, AppType::TUI);
    }

    #[test]
    fn test_app_type_deserialization_gui() {
        let json = r#""GUI""#;
        let app_type: AppType = serde_json::from_str(json).unwrap();
        assert_eq!(app_type, AppType::GUI);
    }

    #[test]
    fn test_app_type_all_variants() {
        let variants = vec![AppType::CLI, AppType::TUI, AppType::GUI];
        assert_eq!(variants.len(), 3);

        for variant in variants {
            let s = variant.as_str();
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn test_app_type_as_str_length() {
        assert_eq!(AppType::CLI.as_str().len(), 3);
        assert_eq!(AppType::TUI.as_str().len(), 3);
        assert_eq!(AppType::GUI.as_str().len(), 3);
    }

    #[test]
    fn test_app_type_match_exhaustive() {
        let app_type = AppType::CLI;
        let result = match app_type {
            AppType::CLI => "matched CLI",
            AppType::TUI => "matched TUI",
            AppType::GUI => "matched GUI",
        };
        assert_eq!(result, "matched CLI");
    }
}
