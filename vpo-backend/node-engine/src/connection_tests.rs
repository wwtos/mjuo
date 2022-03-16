use serde_json::json;

use crate::{connection::Connection, errors::NodeError, node::NodeIndex};

#[test]
fn index_deserialize() {
    let json = json! {{
        "index": 1_i32,
        "generation": 0_i32
    }};

    let json_str = json.to_string();
    println!("{}", json_str);

    let index = serde_json::from_str::<NodeIndex>(json_str.as_str()).unwrap();

    assert_eq!(index, NodeIndex {
        index: 1,
        generation: 0
    });
}

fn socket_type_deserialize() {

}

#[test]
fn connection_deserialize() {
    let json = json! {{
        "from_socket_type": {
            "type": "Stream",
            "content": 
                {
                    "type": "Audio"
                }
            
        },
        "from_node": {
            "index": 0_i32,
            "generation": 0_i32
        },
        "to_socket_type": {
            "type": "Stream",
            "content": 
                {
                    "type": "Audio"
                }
            
        },
        "to_node": {
            "index": 1_i32,
            "generation": 0_i32
        }
    }};

    let json_str = json.to_string();

    println!("{}", json_str);

    let connection: Result<Connection, serde_json::Error> = serde_json::from_str::<Connection>(json_str.as_str());

    println!("{:?}", connection);
}