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
}
