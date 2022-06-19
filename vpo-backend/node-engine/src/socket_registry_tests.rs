use crate::{
    connection::SocketType,
    connection::{MidiSocketType, StreamSocketType},
    errors::NodeError,
    socket_registry::SocketRegistry,
};

#[test]
fn test_request_new_socket_name() {
    let mut registry = SocketRegistry::new();

    let res = registry.register_socket(
        "custom_socket".to_string(),
        SocketType::Stream(StreamSocketType::Audio),
        "Custom socket".to_string(),
        None,
    );

    assert_eq!(
        res.unwrap(),
        SocketType::Stream(StreamSocketType::Dynamic(0))
    )
}

#[test]
fn test_request_existing_socket_name() {
    let mut registry = SocketRegistry::new();

    let res1 = registry.register_socket(
        "custom_socket".to_string(),
        SocketType::Stream(StreamSocketType::Audio),
        "Custom socket".to_string(),
        None,
    );
    let res2 = registry.register_socket(
        "custom_socket".to_string(),
        SocketType::Stream(StreamSocketType::Audio),
        "Custom socket".to_string(),
        None,
    );

    assert_eq!(res1.unwrap(), res2.unwrap())
}

#[test]
fn test_request_existing_socket_name_different_type() {
    let mut registry = SocketRegistry::new();

    registry
        .register_socket(
            "custom_socket".to_string(),
            SocketType::Stream(StreamSocketType::Audio),
            "Custom socket".to_string(),
            None,
        )
        .unwrap();
    let res2 = registry.register_socket(
        "custom_socket".to_string(),
        SocketType::Midi(MidiSocketType::Default),
        "Custom socket".to_string(),
        None,
    );

    assert_eq!(
        format!("{:?}", res2),
        format!(
            "{:?}",
            Result::<(), NodeError>::Err(NodeError::RegistryCollision("custom_socket".to_string()))
        )
    )
}

#[test]
fn test_get_list() {}

#[test]
fn test_get_metadata() {}
