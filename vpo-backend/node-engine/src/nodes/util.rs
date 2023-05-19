use rhai::Dynamic;

#[derive(Debug, Clone)]
pub enum ProcessState<T> {
    Unprocessed(T),
    Processed,
    None,
}

impl<T> ProcessState<T> {
    pub fn as_unprocessed(self) -> Option<T> {
        match self {
            ProcessState::Unprocessed(value) => Some(value),
            _ => None,
        }
    }
}

pub fn value_to_dynamic(value: serde_json::Value) -> Dynamic {
    match value {
        serde_json::Value::Null => Dynamic::from(()),
        serde_json::Value::Bool(value) => Dynamic::from(value),
        serde_json::Value::Number(value) => {
            if value.is_i64() {
                Dynamic::from(value.as_i64().unwrap() as i32)
            } else {
                Dynamic::from(value.as_f64().unwrap() as f32)
            }
        }
        serde_json::Value::String(value) => Dynamic::from(value),
        serde_json::Value::Array(array) => Dynamic::from(array.into_iter().map(value_to_dynamic)),
        serde_json::Value::Object(object) => Dynamic::from(
            object
                .into_iter()
                .map(|(k, v)| (smartstring::SmartString::from(k), value_to_dynamic(v)))
                .collect::<rhai::Map>(),
        ),
    }
}
