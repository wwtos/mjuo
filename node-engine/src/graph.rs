use std::{borrow::Borrow, ops::Deref};

use crate::{
    errors::{Error, ErrorType},
    node::{
        Connection, GenerationalNode, InputSideConnection, NodeIndex, NodeWrapper,
        OutputSideConnection, SocketType,
    },
};

pub struct Graph {
    nodes: Vec<Option<GenerationalNode>>,
}

impl Graph {
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
        if let Some(from_extracted) = self.get_node(from_index) {
            from = from_extracted;
        } else {
            return Err(Error::new(
                format!("`from` node does not exist (index {:?})", from_index),
                ErrorType::NodeDoesNotExist,
            ));
        };

        // does "to" exist?
        if let Some(to_extracted) = self.get_node(to_index) {
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

    pub fn get_node(&self, index: NodeIndex) -> Option<GenerationalNode> {
        // out of bounds?
        if index.index >= self.nodes.len() {
            return None;
        }

        let node = &self.nodes[index.index];

        // node exists there?
        if let Some(node) = node {
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
