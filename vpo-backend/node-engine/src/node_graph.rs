use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use rhai::Engine;
use serde_json::json;

use crate::{
    connection::{
        Connection, InputSideConnection, OutputSideConnection, SocketDirection, SocketType,
    },
    errors::NodeError,
    node::{GenerationalNode, Node, NodeIndex, NodeRow, NodeWrapper},
    nodes::variants::NodeVariant,
    socket_registry::SocketRegistry,
};

#[derive(Debug)]
pub struct NodeGraph {
    nodes: Vec<PossibleNode>,
}

#[derive(Debug)]
pub enum PossibleNode {
    Some(GenerationalNode),
    None(u32), // last generation that was here
}

fn create_new_node(
    node: NodeVariant,
    generation: u32,
    registry: &mut SocketRegistry,
    scripting_engine: &Engine,
) -> PossibleNode {
    PossibleNode::Some(GenerationalNode {
        node: Rc::new(RefCell::new(NodeWrapper::new(
            node,
            NodeIndex {
                index: 0,
                generation: 0,
            },
            registry,
            scripting_engine,
        ))),
        generation,
    })
}

impl NodeGraph {
    pub fn new() -> NodeGraph {
        NodeGraph { nodes: Vec::new() }
    }

    pub fn add_node(
        &mut self,
        node: NodeVariant,
        registry: &mut SocketRegistry,
        scripting_engine: &Engine,
    ) -> NodeIndex {
        let index;
        let new_generation;

        if self.nodes.is_empty() {
            self.nodes
                .push(create_new_node(node, 0, registry, scripting_engine));

            index = self.nodes.len() - 1;
            new_generation = 0;
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

                self.nodes[index] =
                    create_new_node(node, new_generation, registry, scripting_engine);
            } else {
                self.nodes
                    .push(create_new_node(node, 0, registry, scripting_engine));

                index = self.nodes.len() - 1;
                new_generation = 0;
            }
        }

        let full_index = NodeIndex {
            index,
            generation: new_generation,
        };

        let new_node_wrapper = self.get_node(&full_index).unwrap().node;
        let mut new_node = (*new_node_wrapper).borrow_mut();

        new_node.set_index(full_index);

        // now our nodes knows its index and generation, we're all set!
        full_index
    }

    pub fn connect(
        &mut self,
        from_index: NodeIndex,
        from_socket_type: SocketType,
        to_index: NodeIndex,
        to_socket_type: SocketType,
    ) -> Result<Connection, NodeError> {
        // check that the node doesn't have an existing connection of this exact type
        // (one output can be connected to many imputs, one to many)

        let from;
        let to;

        // does "from" exist?
        if let Some(from_extracted) = self.get_node(&from_index) {
            from = from_extracted;
        } else {
            return Err(NodeError::NodeDoesNotExist(from_index));
        };

        // does "to" exist?
        if let Some(to_extracted) = self.get_node(&to_index) {
            to = to_extracted;
        } else {
            return Err(NodeError::NodeDoesNotExist(to_index));
        };

        let mut from = (*from.node).borrow_mut();
        let mut to = (*to.node).borrow_mut();

        // check if "to" is connected from "from"
        if let Some(to_connection) = to.get_input_connection_by_type(&to_socket_type) {
            if to_connection.from_node == from_index {
                return Err(NodeError::AlreadyConnected(
                    from_socket_type,
                    to_socket_type,
                ));
            }

            // it can't be connected twice by anything
            return Err(NodeError::InputSocketOccupied(to_socket_type));
        }

        // make sure `from_type` exists in `from's` outputs
        if !from.has_output_socket(&from_socket_type) {
            return Err(NodeError::SocketDoesNotExist(from_socket_type));
        }

        // make sure `to_type` exists in `to's` inputs
        if !to.has_input_socket(&to_socket_type) {
            return Err(NodeError::SocketDoesNotExist(to_socket_type));
        }

        // make sure the types are of the same family (midi can't connect to stream, etc)
        if mem::discriminant(&from_socket_type) != mem::discriminant(&to_socket_type) {
            return Err(NodeError::IncompatibleSocketTypes(
                from_socket_type,
                to_socket_type,
            ));
        }

        // unless the graph invariant isn't upheld where every connection is referenced both ways
        // (from both connected nodes), we should be good here

        // now we'll create the connection (two-way)
        to.add_input_connection_unsafe(InputSideConnection {
            from_socket_type: from_socket_type.clone(),
            from_node: from.get_index(),
            to_socket_type: to_socket_type.clone(),
        });

        from.add_output_connection_unsafe(OutputSideConnection {
            from_socket_type: from_socket_type.clone(),
            to_node: to.get_index(),
            to_socket_type: to_socket_type.clone(),
        });

        Ok(Connection {
            from_socket_type,
            from_node: from.get_index(),
            to_socket_type,
            to_node: to.get_index(),
        })
    }

    pub fn disconnect(
        &mut self,
        from_index: NodeIndex,
        from_socket_type: SocketType,
        to_index: NodeIndex,
        to_socket_type: SocketType,
    ) -> Result<Connection, NodeError> {
        // check that the connection exists
        let from;
        let to;

        // does "from" exist?
        if let Some(from_extracted) = self.get_node(&from_index) {
            from = from_extracted;
        } else {
            return Err(NodeError::NodeDoesNotExist(from_index));
        };

        // does "to" exist?
        if let Some(to_extracted) = self.get_node(&to_index) {
            to = to_extracted;
        } else {
            return Err(NodeError::NodeDoesNotExist(to_index));
        };

        let mut from = (*from.node).borrow_mut();
        let mut to = (*to.node).borrow_mut();

        // check if "to" is connected from "from"
        let already_connected =
            if let Some(to_connection) = to.get_input_connection_by_type(&to_socket_type) {
                to_connection.from_node == from_index
            } else {
                false
            };

        if !already_connected {
            return Err(NodeError::NotConnected);
        }

        // unless the graph invariant isn't upheld where every connection is referenced both ways
        // (from both connected nodes), we should be good here

        // now we'll remove the connection on both nodes
        to.remove_input_socket_connection_unsafe(&to_socket_type)?;

        from.remove_output_socket_connection_unsafe(&OutputSideConnection {
            from_socket_type: from_socket_type.clone(),
            to_node: to.get_index(),
            to_socket_type: to_socket_type.clone(),
        })?;

        Ok(Connection {
            from_socket_type,
            from_node: from.get_index(),
            to_socket_type,
            to_node: to.get_index(),
        })
    }

    /// Initializes a node
    ///
    /// Returns whether the node has changed itself
    pub fn init_node(
        &mut self,
        index: &NodeIndex,
        socket_registry: &mut SocketRegistry,
        scripting_engine: &Engine,
    ) -> Result<bool, NodeError> {
        let mut has_changed_self = false;

        if let Some(node_ref) = self.get_node(index) {
            let mut node_wrapper = (*node_ref.node).borrow_mut();

            let props = node_wrapper.get_properties().clone();

            let node = &mut node_wrapper.node;
            let init_result = node.init(&props, socket_registry, scripting_engine);

            if init_result.did_rows_change {
                let old_rows = node_wrapper.get_node_rows().clone();
                let new_rows = &init_result.node_rows;

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

                for removed_row in removed_rows {
                    let type_and_direction = removed_row.to_type_and_direction();

                    if let Some(type_and_direction) = type_and_direction {
                        let (socket_type, direction) = type_and_direction;

                        match direction {
                            SocketDirection::Input => {
                                let input_connection =
                                    node_wrapper.get_input_connection_by_type(&socket_type);

                                if let Some(input_connection) = input_connection {
                                    let from_ref = self.get_node(&input_connection.from_node);

                                    if let Some(from_ref) = from_ref {
                                        let mut from_wrapper = (*from_ref.node).borrow_mut();

                                        from_wrapper
                                            .remove_output_socket_connection_unsafe(
                                                &OutputSideConnection {
                                                    from_socket_type: input_connection
                                                        .from_socket_type,
                                                    to_node: *index,
                                                    to_socket_type: input_connection
                                                        .to_socket_type
                                                        .clone(),
                                                },
                                            )
                                            .unwrap();
                                    }

                                    node_wrapper
                                        .remove_input_socket_connection_unsafe(
                                            &input_connection.to_socket_type,
                                        )
                                        .unwrap();
                                }
                            }
                            SocketDirection::Output => {
                                let output_connections =
                                    node_wrapper.get_output_connections_by_type(&socket_type);

                                for output_connection in output_connections {
                                    let to_ref = self.get_node(&output_connection.to_node);

                                    if let Some(to_ref) = to_ref {
                                        let mut to_wrapper = (*to_ref.node).borrow_mut();

                                        // remove the other connection to this one
                                        to_wrapper
                                            .remove_input_socket_connection_unsafe(
                                                &output_connection.to_socket_type,
                                            )
                                            .unwrap();
                                    }

                                    // remove this connection to the other one
                                    node_wrapper
                                        .remove_output_socket_connection_unsafe(
                                            &OutputSideConnection {
                                                from_socket_type: output_connection
                                                    .from_socket_type,
                                                to_node: output_connection.to_node,
                                                to_socket_type: output_connection.to_socket_type,
                                            },
                                        )
                                        .unwrap();
                                }
                            }
                        }
                    }
                }

                // at which point we can _finally_ update the node's row list
                node_wrapper.set_node_rows(init_result.node_rows);
                has_changed_self = true;
            }

            // if the node returned any properties it wanted to change, apply them here
            if let Some(new_props) = init_result.changed_properties {
                for (key, prop) in new_props.into_iter() {
                    node_wrapper.set_property(key, prop);
                }
            }
        }

        Ok(has_changed_self)
    }

    pub fn get_node(&self, index: &NodeIndex) -> Option<GenerationalNode> {
        // out of bounds?
        if index.index >= self.nodes.len() {
            return None;
        }

        let node = &self.nodes[index.index];

        // node exists there?
        if let PossibleNode::Some(node) = node {
            // make sure it's the same generation
            if node.generation != index.generation {
                None
            } else {
                Some(node.clone())
            }
        } else {
            None
        }
    }

    pub fn remove_node(&mut self, index: &NodeIndex) -> Result<(), NodeError> {
        // out of bounds?
        if index.index >= self.nodes.len() {
            return Err(NodeError::IndexOutOfBounds(index.index));
        }

        let node = &self.nodes[index.index];

        let node_to_remove_index;

        // node exists there?
        if let PossibleNode::Some(node) = node {
            // make sure it's the same generation
            if node.generation != index.generation {
                return Err(NodeError::NodeDoesNotExist(*index));
            } else {
                // remove any connected node connections
                let node = (*((*node).node)).borrow();

                for input_socket in node.list_connected_input_sockets() {
                    // follow the input socket
                    let from_node = self.get_node(&input_socket.from_node);

                    if let Some(from_node) = from_node {
                        let from_node = from_node.node;
                        let mut from_node = (*from_node).borrow_mut();

                        from_node.remove_output_socket_connection_unsafe(
                            &OutputSideConnection {
                                from_socket_type: input_socket.from_socket_type,
                                to_node: node.get_index(),
                                to_socket_type: input_socket.to_socket_type,
                            },
                        )?;
                    }
                    // if it doesn't exist, obviously we don't need to worry about removing its connection
                }

                for output_socket in node.list_connected_output_sockets() {
                    // follow the output socket
                    let to_node = self.get_node(&output_socket.to_node);

                    if let Some(to_node) = to_node {
                        let to_node = to_node.node;
                        let mut to_node = (*to_node).borrow_mut();

                        to_node
                            .remove_input_socket_connection_unsafe(&output_socket.to_socket_type)?;
                    }
                    // if it doesn't exist, obviously we don't need to worry about removing its connection
                }

                node_to_remove_index = node.get_index();
            }
        } else {
            return Err(NodeError::NodeDoesNotExist(*index));
        }

        // move down here so the borrow isn't in the scope anymore
        self.nodes[node_to_remove_index.index] =
            PossibleNode::None(node_to_remove_index.generation);

        Ok(())
    }

    pub fn get_nodes(&self) -> &Vec<PossibleNode> {
        &self.nodes
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl NodeGraph {
    pub fn serialize_to_json(&self) -> Result<serde_json::Value, NodeError> {
        // serialize all of the graph nodes, as it currently stands
        let nodes = serde_json::Value::Array(
            self.nodes
                .iter()
                .map(|node| {
                    if let PossibleNode::Some(node) = node {
                        let node = &node.node;
                        let node = (*node).borrow();

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
            if let PossibleNode::Some(some_node) = node {
                let node = &some_node.node;
                let node = (*node).borrow();

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
            .map(serde_json::to_value)
            .collect::<Result<Vec<serde_json::Value>, _>>()?;

        Ok(json!({
            "nodes": nodes,
            "connections": connections
        }))
    }
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new()
    }
}