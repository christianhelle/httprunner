use super::substitution::*;
use crate::types::Variable;

#[test]
fn test_substitute_variables_simple() {
    let variables = vec![Variable {
        name: "host".to_string(),
        value: "api.example.com".to_string(),
    }];

    let result = substitute_variables("https://{{host}}/users", &variables);
    assert_eq!(result, "https://api.example.com/users");
}

#[test]
fn test_substitute_variables_multiple() {
    let variables = vec![
        Variable {
            name: "host".to_string(),
            value: "api.example.com".to_string(),
        },
        Variable {
            name: "port".to_string(),
            value: "8080".to_string(),
        },
    ];

    let result = substitute_variables("https://{{host}}:{{port}}/api", &variables);
    assert_eq!(result, "https://api.example.com:8080/api");
}

#[test]
fn test_substitute_variables_not_found() {
    let variables = vec![Variable {
        name: "host".to_string(),
        value: "api.example.com".to_string(),
    }];

    let result = substitute_variables("https://{{unknown}}/users", &variables);
    assert_eq!(result, "https://{{unknown}}/users");
}

#[test]
fn test_substitute_variables_no_variables() {
    let variables = vec![];

    let result = substitute_variables("https://{{host}}/users", &variables);
    assert_eq!(result, "https://{{host}}/users");
}

#[test]
fn test_substitute_variables_empty_string() {
    let variables = vec![Variable {
        name: "host".to_string(),
        value: "api.example.com".to_string(),
    }];

    let result = substitute_variables("", &variables);
    assert_eq!(result, "");
}

#[test]
fn test_substitute_variables_no_placeholders() {
    let variables = vec![Variable {
        name: "host".to_string(),
        value: "api.example.com".to_string(),
    }];

    let result = substitute_variables("https://example.com/users", &variables);
    assert_eq!(result, "https://example.com/users");
}

#[test]
fn test_substitute_variables_incomplete_braces() {
    let variables = vec![Variable {
        name: "host".to_string(),
        value: "api.example.com".to_string(),
    }];

    let result = substitute_variables("https://{{host/users", &variables);
    assert_eq!(result, "https://{{host/users");
}

#[test]
fn test_substitute_variables_single_braces() {
    let variables = vec![Variable {
        name: "host".to_string(),
        value: "api.example.com".to_string(),
    }];

    let result = substitute_variables("https://{host}/users", &variables);
    assert_eq!(result, "https://{host}/users");
}

#[test]
fn test_substitute_variables_empty_placeholder() {
    let variables = vec![Variable {
        name: "".to_string(),
        value: "value".to_string(),
    }];

    // Empty placeholder matches empty variable name
    let result = substitute_variables("test {{}}", &variables);
    assert_eq!(result, "test value");
}

#[test]
fn test_substitute_variables_same_name_multiple_times() {
    let variables = vec![Variable {
        name: "x".to_string(),
        value: "5".to_string(),
    }];

    let result = substitute_variables("{{x}} + {{x}} = 10", &variables);
    assert_eq!(result, "5 + 5 = 10");
}

#[test]
fn test_substitute_variables_in_json() {
    let variables = vec![
        Variable {
            name: "username".to_string(),
            value: "john".to_string(),
        },
        Variable {
            name: "email".to_string(),
            value: "john@example.com".to_string(),
        },
    ];

    let result = substitute_variables(r#"{"user":"{{username}}","email":"{{email}}"}"#, &variables);
    assert_eq!(result, r#"{"user":"john","email":"john@example.com"}"#);
}

#[test]
fn test_substitute_variables_adjacent_placeholders() {
    let variables = vec![
        Variable {
            name: "first".to_string(),
            value: "Hello".to_string(),
        },
        Variable {
            name: "second".to_string(),
            value: "World".to_string(),
        },
    ];

    let result = substitute_variables("{{first}}{{second}}", &variables);
    assert_eq!(result, "HelloWorld");
}

#[test]
fn test_substitute_variables_with_special_chars_in_value() {
    let variables = vec![Variable {
        name: "password".to_string(),
        value: "p@$$w0rd!".to_string(),
    }];

    let result = substitute_variables("Password: {{password}}", &variables);
    assert_eq!(result, "Password: p@$$w0rd!");
}

#[test]
fn test_substitute_variables_nested_braces_in_name() {
    let variables = vec![Variable {
        name: "var".to_string(),
        value: "value".to_string(),
    }];

    // Should handle braces inside the variable name section
    let result = substitute_variables("{{v}ar}}", &variables);
    // This should not match and preserve the original
    assert_eq!(result, "{{v}ar}}");
}

#[test]
fn test_substitute_variables_whitespace_in_name() {
    let variables = vec![Variable {
        name: "my var".to_string(),
        value: "value".to_string(),
    }];

    let result = substitute_variables("{{my var}}", &variables);
    assert_eq!(result, "value");
}

#[test]
fn test_substitute_variables_case_sensitive() {
    let variables = vec![
        Variable {
            name: "Host".to_string(),
            value: "UPPER".to_string(),
        },
        Variable {
            name: "host".to_string(),
            value: "lower".to_string(),
        },
    ];

    let result = substitute_variables("{{Host}} vs {{host}}", &variables);
    assert_eq!(result, "UPPER vs lower");
}

#[test]
fn test_substitute_variables_numeric_values() {
    let variables = vec![
        Variable {
            name: "port".to_string(),
            value: "8080".to_string(),
        },
        Variable {
            name: "timeout".to_string(),
            value: "30000".to_string(),
        },
    ];

    let result = substitute_variables("Port: {{port}}, Timeout: {{timeout}}ms", &variables);
    assert_eq!(result, "Port: 8080, Timeout: 30000ms");
}
