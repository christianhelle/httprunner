use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Variable {
    pub name: String,
    pub value: String,
}
