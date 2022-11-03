use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "variant", content = "data")]
pub enum PropertyType {
    String,
    Integer,
    Float,
    Bool,
    MultipleChoice(Vec<String>),
    Resource(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    pub namespace: String,
    pub resource: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "variant", content = "data")]
pub enum Property {
    String(String),
    Integer(i32),
    Float(f32),
    Bool(bool),
    MultipleChoice(String),
    Resource(Resource),
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

    pub fn as_resource(self) -> Option<Resource> {
        match self {
            Property::Resource(resource) => Some(resource),
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
