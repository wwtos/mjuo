use std::mem;

use serde::{Deserialize, Serialize};
use serde_json::json;
use snafu::ResultExt;
use sound_engine::SoundConfig;

use crate::{
    connection::{Connection, InputSideConnection, OutputSideConnection, SocketDirection, SocketType},
    errors::{JsonParserSnafu, NodeError, NodeOk, NodeResult, WarningBuilder},
    node::{Node, NodeIndex, NodeInitState, NodeRow, NodeWrapper},
    nodes::variants::NodeVariant,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeGraph {
    nodes: Vec<PossibleNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "variant", content = "data")]
pub enum PossibleNode {
    Some(NodeWrapper, u32),
    None(u32), // last generation that was here
}

pub(crate) fn create_new_node(
    node: NodeVariant,
    index: usize,
    generation: u32,
    state: NodeInitState,
) -> NodeResult<PossibleNode> {
    let new_node = NodeWrapper::new(node, NodeIndex { index, generation }, state)?;

    Ok(NodeOk::new(
        PossibleNode::Some(new_node.value, generation),
        new_node.warnings,
    ))
}

impl NodeGraph {
    pub fn new() -> NodeGraph {
        NodeGraph { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, node: NodeVariant, state: NodeInitState) -> NodeResult<NodeIndex> {
        let index;
        let new_generation;

        let warnings = if self.nodes.is_empty() {
            let new_node = create_new_node(node, 0, 0, state)?;

            self.nodes.push(new_node.value);

            index = self.nodes.len() - 1;
            new_generation = 0;

            new_node.warnings
        } else {
            // find an empty slot (if any)
            let potential_spot = self.nodes.iter().position(|node| {
                // check if the node enum is of type None
                mem::discriminant(node) == mem::discriminant(&PossibleNode::None(0))
            });

            if let Some(i) = potential_spot {
                index = i; // this is where we'll insert the new node

                if let PossibleNode::None(last_generation) = self.nodes[i] {
                    new_generation = last_generation + 1;
                } else {
                    unreachable!(
                        "This is unreachable as we determined \
                    just above in the `position` method that the node at \
                    this location was PossibleNode::None"
                    );
                };

                let new_node = create_new_node(node, index, new_generation, state)?;
                self.nodes[index] = new_node.value;

                new_node.warnings
            } else {
                index = self.nodes.len();
                new_generation = 0;

                let new_node = create_new_node(node, index, new_generation, state)?;
                self.nodes.push(new_node.value);

                new_node.warnings
            }
        };

        let full_index = NodeIndex {
            index,
            generation: new_generation,
        };

        // now our nodes knows its index and generation, we're all set!
        Ok(NodeOk::new(full_index, warnings))
    }

    pub fn connect(
        &mut self,
        from_index: &NodeIndex,
        from_socket_type: &SocketType,
        to_index: &NodeIndex,
        to_socket_type: &SocketType,
    ) -> Result<Connection, NodeError> {
        // check that the node doesn't have an existing connection of this exact type
        // (one output can be connected to many imputs, one to many)

        // does "from" exist?
        let from = if let Some(from_wrapper) = self.get_node(from_index) {
            from_wrapper
        } else {
            return Err(NodeError::NodeDoesNotExist {
                node_index: *from_index,
            });
        };

        // does "to" exist?
        let to = if let Some(to_wrapper) = self.get_node(to_index) {
            to_wrapper
        } else {
            return Err(NodeError::NodeDoesNotExist { node_index: *to_index });
        };

        // check if "to" is connected from "from"
        if let Some(to_connection) = to.get_input_connection_by_type(to_socket_type) {
            if &to_connection.from_node == from_index {
                return Err(NodeError::AlreadyConnected {
                    from: from_socket_type.clone(),
                    to: to_socket_type.clone(),
                });
            }

            // it can't be connected twice by anything
            return Err(NodeError::InputSocketOccupied {
                socket_type: to_socket_type.clone(),
            });
        }

        // make sure `from_type` exists in `from's` outputs
        if !from.has_output_socket(from_socket_type) {
            return Err(NodeError::SocketDoesNotExist {
                socket_type: from_socket_type.clone(),
            });
        }

        // make sure `to_type` exists in `to's` inputs
        if !to.has_input_socket(to_socket_type) {
            return Err(NodeError::SocketDoesNotExist {
                socket_type: to_socket_type.clone(),
            });
        }

        // make sure the types are of the same family (midi can't connect to stream, etc)
        if mem::discriminant(from_socket_type) != mem::discriminant(to_socket_type) {
            return Err(NodeError::IncompatibleSocketTypes {
                from: from_socket_type.clone(),
                to: to_socket_type.clone(),
            });
        }

        // unless the graph invariant isn't upheld where every connection is referenced both ways
        // (from both connected nodes), we should be good here

        // now we'll create the connection (two-way)
        self.get_node_mut(to_index)
            .unwrap()
            .add_input_connection_unchecked(InputSideConnection {
                from_socket_type: from_socket_type.clone(),
                from_node: *from_index,
                to_socket_type: to_socket_type.clone(),
            });

        self.get_node_mut(from_index)
            .unwrap()
            .add_output_connection_unchecked(OutputSideConnection {
                from_socket_type: from_socket_type.clone(),
                to_node: *to_index,
                to_socket_type: to_socket_type.clone(),
            });

        Ok(Connection {
            from_socket_type: from_socket_type.clone(),
            from_node: *from_index,
            to_socket_type: to_socket_type.clone(),
            to_node: *to_index,
        })
    }

    pub fn disconnect(
        &mut self,
        from_index: &NodeIndex,
        from_socket_type: &SocketType,
        to_index: &NodeIndex,
        to_socket_type: &SocketType,
    ) -> Result<Connection, NodeError> {
        // check that the connection exists

        // does "from" exist?
        if self.get_node(from_index).is_none() {
            return Err(NodeError::NodeDoesNotExist {
                node_index: *from_index,
            });
        }

        // does "to" exist?
        let to = if let Some(to_wrapper) = self.get_node(to_index) {
            to_wrapper
        } else {
            return Err(NodeError::NodeDoesNotExist { node_index: *to_index });
        };

        // check if "to" is connected from "from"
        let already_connected = if let Some(to_connection) = to.get_input_connection_by_type(to_socket_type) {
            &to_connection.from_node == from_index
        } else {
            false
        };

        if !already_connected {
            return Err(NodeError::NotConnected);
        }

        // unless the graph invariant isn't upheld where every connection is referenced both ways
        // (from both connected nodes), we should be good here

        // now we'll remove the connection on both nodes
        self.get_node_mut(to_index)
            .unwrap()
            .remove_input_socket_connection_unchecked(to_socket_type)?;

        self.get_node_mut(from_index)
            .unwrap()
            .remove_output_socket_connection_unchecked(&OutputSideConnection {
                from_socket_type: from_socket_type.clone(),
                to_node: *to_index,
                to_socket_type: to_socket_type.clone(),
            })?;

        Ok(Connection {
            from_socket_type: from_socket_type.clone(),
            from_node: *from_index,
            to_socket_type: to_socket_type.clone(),
            to_node: *to_index,
        })
    }

    /// Initializes a node
    ///
    /// Returns whether the node has changed itself
    pub fn init_node(
        &mut self,
        index: &NodeIndex,
        state: NodeInitState,
        force_update: bool,
    ) -> Result<NodeOk<bool>, NodeError> {
        let mut has_changed_self = false;
        let mut warnings = WarningBuilder::new();

        // will return the new node rows, if they changed
        let possible_rows = if let Some(node_wrapper) = self.get_node_mut(index) {
            let NodeInitState {
                props: _,
                registry,
                script_engine,
                global_state,
            } = state;

            let props = node_wrapper.get_properties().clone();

            let node = &mut node_wrapper.node;
            let init_result = node.init(NodeInitState {
                props: &props,
                registry,
                script_engine,
                global_state,
            })?;

            warnings.append_warnings(init_result.warnings);

            // if the node returned any properties it wanted to change, apply them here
            if let Some(new_props) = init_result.value.changed_properties {
                for (key, prop) in new_props.into_iter() {
                    node_wrapper.set_property(key, prop);
                }
            }

            // return a list of all the rows that changed to the outer scope
            if init_result.value.did_rows_change || force_update {
                let old_rows = node_wrapper.get_node_rows().clone();
                let new_rows = &init_result.value.node_rows;

                // TODO: implement sockets changing properly
                // aka, if a socket is removed, safely disconnect it from the
                // other node

                // The main thing here is to see what properties were removed -- if it was removed
                // it needs to be disconnected safely
                let removed_rows: Vec<NodeRow> = old_rows
                    .iter()
                    .filter(|old_row| {
                        // if it's not in the new row, but it was in the old row,
                        // it's been removed
                        !new_rows.iter().any(|new_row| new_row == *old_row)
                    })
                    .cloned()
                    .collect();

                Some((removed_rows, init_result.value.node_rows))
            } else {
                None
            }
        } else {
            return Err(NodeError::NodeDoesNotExist { node_index: *index });
        };

        if let Some((removed_rows, new_node_rows)) = possible_rows {
            for removed_row in removed_rows {
                let type_and_direction = removed_row.to_type_and_direction();

                if let Some(type_and_direction) = type_and_direction {
                    let (socket_type, direction) = type_and_direction;

                    match direction {
                        SocketDirection::Input => {
                            let node_wrapper = self.get_node(index).unwrap();
                            let input_connection = node_wrapper.get_input_connection_by_type(&socket_type);

                            if let Some(input_connection) = input_connection {
                                let from_wrapper = self.get_node_mut(&input_connection.from_node);

                                if let Some(from_wrapper) = from_wrapper {
                                    from_wrapper.remove_output_socket_connection_unchecked(&OutputSideConnection {
                                        from_socket_type: input_connection.from_socket_type,
                                        to_node: *index,
                                        to_socket_type: input_connection.to_socket_type.clone(),
                                    })?;
                                }

                                self.get_node_mut(index)
                                    .unwrap()
                                    .remove_input_socket_connection_unchecked(&input_connection.to_socket_type)?;
                            }
                        }
                        SocketDirection::Output => {
                            let node_wrapper = self.get_node(index).unwrap();
                            let output_connections = node_wrapper.get_output_connections_by_type(&socket_type);

                            for output_connection in output_connections {
                                let to_wrapper = self.get_node_mut(&output_connection.to_node);

                                if let Some(to_wrapper) = to_wrapper {
                                    // remove the other connection to this one
                                    to_wrapper
                                        .remove_input_socket_connection_unchecked(&output_connection.to_socket_type)?;
                                }

                                // remove this connection to the other one
                                self.get_node_mut(index)
                                    .unwrap()
                                    .remove_output_socket_connection_unchecked(&OutputSideConnection {
                                        from_socket_type: output_connection.from_socket_type,
                                        to_node: output_connection.to_node,
                                        to_socket_type: output_connection.to_socket_type,
                                    })?;
                            }
                        }
                    }
                }
            }

            // at which point we can _finally_ update the node's row list
            self.get_node_mut(index).unwrap().set_node_rows(new_node_rows);
            has_changed_self = true;
        }

        Ok(NodeOk::new(has_changed_self, warnings.into_warnings()))
    }

    pub fn get_node(&self, index: &NodeIndex) -> Option<&NodeWrapper> {
        // out of bounds?
        if index.index >= self.nodes.len() {
            return None;
        }

        let node = &self.nodes[index.index];

        // node exists there?
        if let PossibleNode::Some(node, generation) = node {
            // make sure it's the same generation
            if generation != &index.generation {
                None
            } else {
                Some(node)
            }
        } else {
            None
        }
    }

    pub fn get_node_mut(&mut self, index: &NodeIndex) -> Option<&mut NodeWrapper> {
        // out of bounds?
        if index.index >= self.nodes.len() {
            return None;
        }

        let node = &mut self.nodes[index.index];

        // node exists there?
        if let PossibleNode::Some(node, generation) = node {
            // make sure it's the same generation
            if generation != &index.generation {
                None
            } else {
                Some(node)
            }
        } else {
            None
        }
    }

    pub fn remove_node(&mut self, index: &NodeIndex) -> Result<(), NodeError> {
        // out of bounds?
        if index.index >= self.nodes.len() {
            return Err(NodeError::IndexOutOfBounds { index: index.index });
        }

        let node = &self.nodes[index.index];

        // node exists there?
        let (input_sockets, output_sockets) = if let PossibleNode::Some(node, generation) = node {
            // make sure it's the same generation
            if generation != &index.generation {
                return Err(NodeError::NodeDoesNotExist { node_index: *index });
            } else {
                (
                    node.list_connected_input_sockets(),
                    node.list_connected_output_sockets(),
                )
            }
        } else {
            return Err(NodeError::NodeDoesNotExist { node_index: *index });
        };

        // remove any connected node connections
        for input_socket in input_sockets {
            // follow the input socket
            let from_wrapper = self.get_node_mut(&input_socket.from_node);

            if let Some(from_wrapper) = from_wrapper {
                from_wrapper.remove_output_socket_connection_unchecked(&OutputSideConnection {
                    from_socket_type: input_socket.from_socket_type,
                    to_node: *index,
                    to_socket_type: input_socket.to_socket_type,
                })?;
            }
            // if it doesn't exist, obviously we don't need to worry about removing its connection
        }

        for output_socket in output_sockets {
            // follow the output socket
            let to_wrapper = self.get_node_mut(&output_socket.to_node);

            if let Some(to_wrapper) = to_wrapper {
                to_wrapper.remove_input_socket_connection_unchecked(&output_socket.to_socket_type)?;
            }
            // if it doesn't exist, obviously we don't need to worry about removing its connection
        }

        self.nodes[index.index] = PossibleNode::None(index.generation);

        Ok(())
    }

    pub fn get_nodes(&self) -> &Vec<PossibleNode> {
        &self.nodes
    }

    pub fn get_nodes_mut(&mut self) -> &mut Vec<PossibleNode> {
        &mut self.nodes
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn set_node_unchecked(&mut self, index: NodeIndex, node: PossibleNode) {
        self.nodes[index.index] = node;
    }
}

impl NodeGraph {
    pub fn serialize_to_json(&self) -> Result<serde_json::Value, NodeError> {
        // serialize all of the graph nodes, as it currently stands
        let nodes = serde_json::Value::Array(
            self.nodes
                .iter()
                .map(|node| {
                    if let PossibleNode::Some(node, _) = node {
                        match node.serialize_to_json() {
                            Ok(json) => json,
                            Err(_) => serde_json::Value::Null,
                        }
                    } else {
                        serde_json::Value::Null
                    }
                })
                .collect::<Vec<serde_json::Value>>(),
        );

        let mut connections: Vec<Connection> = Vec::new();

        // make a list of connections based on the input node, as that can't be connected to multiple things
        for node in &self.nodes {
            if let PossibleNode::Some(node, _) = node {
                let input_sockets = node.list_connected_input_sockets();

                for socket in input_sockets {
                    connections.push(Connection {
                        from_socket_type: socket.from_socket_type,
                        from_node: socket.from_node,
                        to_socket_type: socket.to_socket_type,
                        to_node: node.get_index(),
                    });
                }
            }
        }

        let connections = connections
            .into_iter()
            .map(|x| serde_json::to_value(x).context(JsonParserSnafu))
            .collect::<Result<Vec<serde_json::Value>, _>>()?;

        Ok(json!({
            "nodes": nodes,
            "connections": connections
        }))
    }

    pub fn post_deserialization(&mut self, state: NodeInitState, sound_config: &SoundConfig) -> Result<(), NodeError> {
        let NodeInitState {
            props,
            registry,
            script_engine,
            global_state,
        } = state;

        // go through and run post_deserialization on each node
        // then, initialize all those nodes in the graph here
        self.nodes
            .iter_mut()
            .filter_map(|possible_node| {
                if let PossibleNode::Some(node, _) = possible_node {
                    let res = node.post_deserialization(sound_config);

                    match res {
                        Ok(_) => Some(Ok(node.get_index())),
                        Err(err) => Some(Err(err)),
                    }
                } else {
                    None
                }
            })
            .collect::<Result<Vec<NodeIndex>, NodeError>>()?
            .iter()
            .map(|node_index| {
                self.init_node(
                    node_index,
                    NodeInitState {
                        props,
                        registry,
                        script_engine,
                        global_state,
                    },
                    false,
                )
            })
            .collect::<Result<Vec<_>, NodeError>>()?;

        Ok(())
    }
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new()
    }
}
