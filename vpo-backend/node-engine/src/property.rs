use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum PropertyType {
    String,
    Integer,
    Float,
    Bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Property {
    String(String),
    Integer(i32),
    Float(f32),
    Bool(bool)
}
