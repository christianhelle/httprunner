use crate::types::Variable;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_environment_file(
    http_file_path: &str,
    environment_name: Option<&str>,
) -> Result<Vec<Variable>> {
    let mut variables = Vec::new();

    // Early return if no environment name is provided
    let env_name = match environment_name {
        Some(name) => name,
        None => return Ok(variables),
    };

    // Find the environment file
    let env_file_path = match find_environment_file(http_file_path)? {
        Some(path) => path,
        None => return Ok(variables),
    };

    // Parse the environment configuration
    let env_config = parse_environment_file(&env_file_path)?;

    // Load variables for the specified environment
    if let Some(env_vars) = env_config.get(env_name) {
        for (name, value) in env_vars {
            variables.push(Variable {
                name: name.clone(),
                value: value.clone(),
            });
        }
    }

    Ok(variables)
}

/// Find the http-client.env.json file by searching up the directory tree
///
/// Starts from the directory containing the HTTP file and searches upward
/// until an http-client.env.json file is found or the root directory is reached.
///
/// # Arguments
///
/// * `http_file_path` - Path to the HTTP file to start searching from
///
/// # Returns
///
/// Returns `Some(PathBuf)` if the environment file is found, `None` otherwise
pub(crate) fn find_environment_file(http_file_path: &str) -> Result<Option<PathBuf>> {
    let path = Path::new(http_file_path);
    let mut current_dir = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    loop {
        let env_file = current_dir.join("http-client.env.json");
        if env_file.exists() {
            return Ok(Some(env_file));
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    Ok(None)
}

/// Parse the http-client.env.json file
///
/// Parses a JSON file containing environment configurations with the format:
/// ```json
/// {
///   "dev": {
///     "api_url": "https://dev.api.example.com",
///     "api_key": "dev-key"
///   },
///   "prod": {
///     "api_url": "https://api.example.com",
///     "api_key": "prod-key"
///   }
/// }
/// ```
///
/// # Arguments
///
/// * `file_path` - Path to the environment JSON file
///
/// # Returns
///
/// Returns a HashMap mapping environment names to their variable maps
///
/// # Errors
///
/// Returns an error if the file cannot be read or contains invalid JSON
pub(crate) fn parse_environment_file(
    file_path: &Path,
) -> Result<HashMap<String, HashMap<String, String>>> {
    let content = fs::read_to_string(file_path)?;
    let json: Value = serde_json::from_str(&content)?;

    let mut config = HashMap::new();

    if let Value::Object(root) = json {
        for (env_name, env_value) in root {
            let mut env_vars = HashMap::new();

            if let Value::Object(vars) = env_value {
                for (var_name, var_value) in vars {
                    let value_str = match var_value {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => String::new(),
                        // For objects and arrays, convert to JSON string
                        _ => var_value.to_string(),
                    };
                    env_vars.insert(var_name.clone(), value_str);
                }
            }

            config.insert(env_name.clone(), env_vars);
        }
    }

    Ok(config)
}
