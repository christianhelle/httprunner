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

fn find_environment_file(http_file_path: &str) -> Result<Option<PathBuf>> {
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

fn parse_environment_file(file_path: &Path) -> Result<HashMap<String, HashMap<String, String>>> {
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
                        _ => String::new(),
                    };
                    env_vars.insert(var_name.clone(), value_str);
                }
            }

            config.insert(env_name.clone(), env_vars);
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn load_environment_file_returns_empty_without_environment() {
        let temp = tempdir().unwrap();
        let http_file = temp.path().join("request.http");
        let vars = load_environment_file(http_file.to_str().unwrap(), None).unwrap();
        assert!(vars.is_empty());
    }

    #[test]
    fn load_environment_file_reads_nearest_env_file() {
        let temp = tempdir().unwrap();
        let nested = temp.path().join("nested");
        fs::create_dir(&nested).unwrap();

        let http_file = nested.join("request.http");
        fs::write(&http_file, "GET http://example.com").unwrap();
        fs::write(
            temp.path().join("http-client.env.json"),
            r#"{"dev":{"TOKEN":"abc","COUNT":1,"FLAG":true}}"#,
        )
        .unwrap();

        let vars = load_environment_file(http_file.to_str().unwrap(), Some("dev")).unwrap();
        let map: std::collections::HashMap<_, _> =
            vars.into_iter().map(|v| (v.name, v.value)).collect();
        assert_eq!(map.get("TOKEN"), Some(&"abc".to_string()));
        assert_eq!(map.get("COUNT"), Some(&String::new()));
        assert_eq!(map.get("FLAG"), Some(&String::new()));
    }

    #[test]
    fn find_environment_file_returns_none_when_absent() {
        let temp = tempdir().unwrap();
        let http_file = temp.path().join("request.http");
        fs::write(&http_file, "GET http://example.com").unwrap();
        let found = find_environment_file(http_file.to_str().unwrap()).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn parse_environment_file_handles_non_string_values() {
        let temp = tempdir().unwrap();
        let env_file = temp.path().join("http-client.env.json");
        fs::write(
            &env_file,
            r#"{"dev":{"TEXT":"value","NUMBER":123,"OBJECT":{"foo":"bar"}}}"#,
        )
        .unwrap();

        let parsed = parse_environment_file(&env_file).unwrap();
        let dev = parsed.get("dev").unwrap();
        assert_eq!(dev.get("TEXT").unwrap(), "value");
        assert_eq!(dev.get("NUMBER").unwrap(), "");
        assert_eq!(dev.get("OBJECT").unwrap(), "");
    }
}
