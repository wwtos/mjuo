use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use serde_json::json;

use crate::{
    connection::{Connection, InputSideConnection, OutputSideConnection, SocketType},
    errors::{Error, ErrorType},
    node::{GenerationalNode, Node, NodeIndex, NodeWrapper},
};

#[derive(Debug)]
pub struct Graph {
    nodes: Vec<PossibleNode>,
}

#[derive(Debug)]
pub enum PossibleNode {
    Some(GenerationalNode),
    None(u32), // last generation that was here
}

fn create_new_node(node: Box<dyn Node>, generation: u32) -> PossibleNode {
    PossibleNode::Some(GenerationalNode {
        node: Rc::new(RefCell::new(NodeWrapper::new(
            node,
            NodeIndex {
                index: 0,
                generation: 0,
            },
        ))),
        generation,
    })
}

impl Graph {
    pub fn new() -> Graph {
        Graph { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, node: Box<dyn Node>) -> NodeIndex {
        let index;
        let new_generation;

        if self.nodes.is_empty() {
            self.nodes.push(create_new_node(node, 0));

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

                self.nodes[index] = create_new_node(node, new_generation);
            } else {
                self.nodes.push(create_new_node(node, 0));

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
        from_type: SocketType,
        to_index: NodeIndex,
        to_type: SocketType,
    ) -> Result<Connection, Error> {
        // check that the node doesn't have an existing connection of this exact type
        // (one output can be connected to many imputs, one to many)

        let from;
        let to;

        // does "from" exist?
        if let Some(from_extracted) = self.get_node(&from_index) {
            from = from_extracted;
        } else {
            return Err(Error::new(
                format!("`from` node does not exist (index {:?})", from_index),
                ErrorType::NodeDoesNotExist,
            ));
        };

        // does "to" exist?
        if let Some(to_extracted) = self.get_node(&to_index) {
            to = to_extracted;
        } else {
            return Err(Error::new(
                format!("`to` node does not exist (index {:?})", to_index),
                ErrorType::NodeDoesNotExist,
            ));
        };

        let mut from = (*from.node).borrow_mut();
        let mut to = (*to.node).borrow_mut();

        // check if "to" is connected from "from"
        if let Some(to_connection) = to.get_input_connection_by_type(&to_type) {
            if to_connection.from_node == from_index {
                return Err(Error::new(
                    format!(
                        "Connection between {:?} and {:?} already exists",
                        from_type, to_type
                    ),
                    ErrorType::AlreadyConnected,
                ));
            }
        };

        // make sure `from_type` exists in `from's` outputs
        if !from.has_output_socket(&from_type) {
            return Err(Error::new(
                format!("From node does not have socket type {:?}", from_type),
                ErrorType::SocketDoesNotExist,
            ));
        }

        // make sure `to_type` exists in `to's` inputs
        if !to.has_input_socket(&to_type) {
            return Err(Error::new(
                format!("To node does not have socket type {:?}", to_type),
                ErrorType::SocketDoesNotExist,
            ));
        }

        // make sure the types are of the same family (midi can't connect to stream, etc)
        if mem::discriminant(&from_type) != mem::discriminant(&to_type) {
            return Err(Error::new(
                format!("{:?} and {:?} sockets are incompatible", to_type, from_type),
                ErrorType::IncompatibleSocketTypes,
            ));
        }

        // unless the graph invariant isn't upheld where every connection is referenced both ways
        // (from both connected nodes), we should be good here

        // now we'll create the connection (two-way)
        to.add_input_connection_unsafe(InputSideConnection {
            from_socket_type: from_type.clone(),
            from_node: from.get_index(),
            to_socket_type: to_type.clone(),
        });

        from.add_output_connection_unsafe(OutputSideConnection {
            from_socket_type: from_type.clone(),
            to_node: to.get_index(),
            to_socket_type: to_type.clone(),
        });

        Ok(Connection {
            from_socket_type: from_type,
            from_node: from.get_index(),
            to_socket_type: to_type,
            to_node: to.get_index(),
        })
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

    pub fn remove_node(&mut self, index: &NodeIndex) -> Result<(), Error> {
        // out of bounds?
        if index.index >= self.nodes.len() {
            return Err(Error::new(
                format!("Index {} out of bounds!", index.index),
                ErrorType::IndexOutOfBounds,
            ));
        }

        let node = &self.nodes[index.index];

        let node_to_remove_index;

        // node exists there?
        if let PossibleNode::Some(node) = node {
            // make sure it's the same generation
            if node.generation != index.generation {
                return Err(Error::new(
                    format!(
                        "Node at index {} does not exist! (wrong generation)",
                        index.index
                    ),
                    ErrorType::NodeDoesNotExist,
                ));
            } else {
                // remove any connected node connections
                let node = (*((*node).node)).borrow();

                for input_socket in node.list_input_sockets() {
                    // follow the input socket
                    let from_node = self.get_node(&input_socket.from_node);

                    if let Some(from_node) = from_node {
                        let from_node = from_node.node;
                        let mut from_node = (*from_node).borrow_mut();

                        from_node.remove_output_socket_connection(
                            &input_socket.from_socket_type,
                            &node.get_index(),
                            &input_socket.to_socket_type,
                        )?;
                    }
                    // if it doesn't exist, obviously we don't need to worry about removing its connection
                }

                for output_socket in node.list_output_sockets() {
                    // follow the output socket
                    let to_node = self.get_node(&output_socket.to_node);

                    if let Some(to_node) = to_node {
                        let to_node = to_node.node;
                        let mut to_node = (*to_node).borrow_mut();

                        to_node.remove_input_socket_connection(&output_socket.to_socket_type)?;
                    }
                    // if it doesn't exist, obviously we don't need to worry about removing its connection
                }

                node_to_remove_index = node.get_index();
            }
        } else {
            return Err(Error::new(
                format!("Node at index {} does not exist!", index.index),
                ErrorType::NodeDoesNotExist,
            ));
        }

        // move down here so the borrow isn't in the scope anymore
        self.nodes[node_to_remove_index.index] =
            PossibleNode::None(node_to_remove_index.generation);

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

impl Graph {
    pub fn serialize(&self) -> Result<serde_json::Value, Error> {
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

                let input_sockets = node.list_input_sockets();

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
            .map(|connection| connection.serialize_to_json())
            .collect::<Result<Vec<serde_json::Value>, _>>()?;

        Ok(json!({
            "nodes": nodes,
            "connections": connections
        }))
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}