use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum PropertyType {
    String,
    Integer,
    Float,
    Bool,
    MultipleChoice(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Property {
    String(String),
    Integer(i32),
    Float(f32),
    Bool(bool),
    MultipleChoice(String),
}

impl Property {
    pub fn as_string(self) -> Option<String> {
        match self {
            Property::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_float(self) -> Option<f32> {
        match self {
            Property::Float(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_multiple_choice(self) -> Option<String> {
        match self {
            Property::MultipleChoice(value) => Some(value),
            _ => None,
        }
    }
}
