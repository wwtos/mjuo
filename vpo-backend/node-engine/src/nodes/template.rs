use std::collections::HashMap;

use crate::connection::{SocketType, StreamSocketType};
use crate::node::Node;
use crate::property::{Property, PropertyType};

#[derive(Debug)]
pub struct TemplateNode {
    some_stream: f32,
    some_prop: f32
}

impl Default for TemplateNode {
    fn default() -> Self {
        TemplateNode {
            some_stream: 0.0,
            some_prop: 1.0
        }
    }
}

impl Node for TemplateNode {
    fn accept_stream_input(&mut self, socket_type: StreamSocketType, value: f32) {
        if socket_type == StreamSocketType::Audio {
            self.some_stream = value;
        }
    }

    fn get_stream_output(&self, socket_type: StreamSocketType) -> f32 {
        if socket_type == StreamSocketType::Audio {
            self.some_stream
        } else {
            0.0
        }
    }

    fn init(
        &mut self,
        properties: &HashMap<String, Property>,
    ) -> (bool, Option<HashMap<String, Property>>) {
        if let Some(some_prop) = properties.get("some_prop") {
            if let Property::Float(some_extracted_prop) = some_prop {
                self.some_prop = *some_extracted_prop;
            }
        }

        (false, None)
    }

    fn list_input_sockets(&self) -> Vec<SocketType> {
        vec![
            SocketType::Stream(StreamSocketType::Audio),
        ]
    }

    fn list_output_sockets(&self) -> Vec<SocketType> {
        vec![
            SocketType::Stream(StreamSocketType::Audio),
        ]
    }

    fn list_properties(&self) -> HashMap<String, PropertyType> {
        let mut props = HashMap::new();

        props.insert("some_prop".to_string(), PropertyType::Float);

        props
    }
}
