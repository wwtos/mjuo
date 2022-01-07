use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

use crate::{
    errors::{Error, ErrorType},
    node::{
        Connection, GenerationalNode, InputSideConnection, Node, NodeIndex, NodeWrapper,
        OutputSideConnection, SocketType,
    },
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
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}
