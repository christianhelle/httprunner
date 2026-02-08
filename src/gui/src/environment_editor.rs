use iced::{
    widget::{column, scrollable, text, Column},
    Element, Length,
};
use std::collections::HashMap;

use crate::app::Message;

/// Environment editor component for viewing environment variables
pub struct EnvironmentEditor {
    /// The full environment config: env_name -> { var_name -> var_value }
    config: HashMap<String, HashMap<String, String>>,
}

impl EnvironmentEditor {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut col = Column::new().spacing(10).padding(10);

        col = col.push(text("Environment Variables").size(20));

        if self.config.is_empty() {
            col = col.push(text("No environment variables configured."));
            col = col.push(text("Create an http-client.env.json file to define environments."));
        } else {
            for (env_name, variables) in &self.config {
                col = col.push(text(format!("Environment: {}", env_name)).size(16));
                
                if variables.is_empty() {
                    col = col.push(text("  (no variables)"));
                } else {
                    for (key, value) in variables {
                        col = col.push(text(format!("  {} = {}", key, value)));
                    }
                }
                
                col = col.push(text("───────────────"));
            }
        }

        scrollable(col).into()
    }
}

impl Default for EnvironmentEditor {
    fn default() -> Self {
        Self::new()
    }
}
