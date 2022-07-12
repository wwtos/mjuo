use async_std::channel::Sender;
use ipc::ipc_message::IPCMessage;
use node_engine::{
    errors::NodeError,
    graph_manager::{GraphIndex, GraphManager},
    nodes::variants::new_variant,
    socket_registry::SocketRegistry,
};
use rhai::Engine;
use serde_json::{Map, Value};
use sound_engine::SoundConfig;

use crate::{util::update_graph, RouteReturn};

/// this function creates a new node in the graph based on the provided data
///
/// JSON should be formatted thus:
/// ```json
/// {
///     "action": "graph/newNode",
///     "payload": {
///         "type": "[node type]",
///         "ui_data": {
///             foo: "override ui_data here"
///         }
///     }
/// }```
///
pub fn route(
    message: Map<String, Value>,
    current_graph_index: GraphIndex,
    graph_manager: &mut GraphManager,
    to_server: &Sender<IPCMessage>,
    sound_config: &SoundConfig,
    socket_registry: &mut SocketRegistry,
    scripting_engine: &Engine,
) -> Result<Option<RouteReturn>, NodeError> {
    let (index, needs_graph) = if let Value::String(node_type) = &message["payload"]["type"] {
        let graph = &mut graph_manager.get_graph_wrapper_mut(current_graph_index).unwrap().graph;

        let new_node = new_variant(node_type, sound_config).unwrap();

        let new_node_index = graph.add_node(new_node, socket_registry, scripting_engine);
        let new_node_wrapper = graph.get_node(&new_node_index).unwrap();

        (new_node_index, new_node_wrapper.does_need_inner_graph_created())
    } else {
        return Ok(None);
    };

    if needs_graph {
        // create a graph for it
        let new_graph_index = graph_manager.new_graph();

        let graph = &mut graph_manager.get_graph_wrapper_mut(current_graph_index).unwrap().graph;
        let new_node = graph.get_node_mut(&index).unwrap();
        
        new_node.set_inner_graph_index(new_graph_index);
    }

    let graph = &mut graph_manager.get_graph_wrapper_mut(current_graph_index).unwrap().graph;
    if let Value::Object(ui_data) = &message["payload"]["ui_data"] {
        let node = graph.get_node_mut(&index).unwrap();

        // overwrite default values
        for (key, value) in ui_data.to_owned().into_iter() {
            node.set_ui_data_property(key, value);
        }
    }

    update_graph(graph, current_graph_index, to_server);

    Ok(Some(RouteReturn {
        should_reindex_graph: true,
        new_graph_index: None,
    }))
}
