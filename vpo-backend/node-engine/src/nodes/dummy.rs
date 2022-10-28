use crate::{
    errors::{NodeError, NodeOk},
    node::{InitResult, Node, NodeInitState},
};

#[derive(Debug, Default, Clone)]
pub struct DummyNode {}

impl Node for DummyNode {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        InitResult::simple(vec![])
    }
}
