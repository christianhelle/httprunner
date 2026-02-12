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

    if environment_name.is_none() {
        return Ok(variables);
    }

    let env_file_path = find_environment_file(http_file_path)?;
    if env_file_path.is_none() {
        return Ok(variables);
    }

    let env_config = parse_environment_file(&env_file_path.unwrap())?;

    if let Some(env_vars) = env_config.get(environment_name.unwrap()) {
        for (name, value) in env_vars {
            variables.push(Variable {
                name: name.clone(),
                value: value.clone(),
            });
        }
    }

    Ok(variables)
}

pub fn find_environment_file(http_file_path: &str) -> Result<Option<PathBuf>> {
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

pub fn parse_environment_file(
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

pub fn save_environment_file(
    file_path: &Path,
    config: &HashMap<String, HashMap<String, String>>,
) -> Result<()> {
    // Build a sorted JSON object for deterministic output
    let mut root = serde_json::Map::new();
    let mut env_names: Vec<&String> = config.keys().collect();
    env_names.sort();

    for env_name in env_names {
        let vars = &config[env_name];
        let mut env_obj = serde_json::Map::new();
        let mut var_names: Vec<&String> = vars.keys().collect();
        var_names.sort();
        for var_name in var_names {
            env_obj.insert(var_name.clone(), Value::String(vars[var_name].clone()));
        }
        root.insert(env_name.clone(), Value::Object(env_obj));
    }

    let json = serde_json::to_string_pretty(&root)?;
    fs::write(file_path, json)?;
    Ok(())
}
