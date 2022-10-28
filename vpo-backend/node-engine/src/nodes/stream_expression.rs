use rhai::{Scope, AST};
use serde_json::json;
use smallvec::{smallvec, SmallVec};

use crate::connection::{SocketType, StreamSocketType};
use crate::errors::{NodeError, NodeOk, NodeWarning, WarningBuilder};
use crate::node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow};
use crate::property::{Property, PropertyType};

#[derive(Debug, Clone)]
pub struct StreamExpressionNode {
    ast: Option<AST>,
    scope: Scope<'static>,
    values_in: SmallVec<[f32; 8]>,
    values_in_mapping: SmallVec<[(u64, usize); 8]>,
    value_out: f32,
}

impl StreamExpressionNode {
    pub fn new() -> StreamExpressionNode {
        StreamExpressionNode {
            scope: Scope::new(),
            ast: None,
            values_in: smallvec![],
            values_in_mapping: smallvec![],
            value_out: 0.0,
        }
    }
}

impl Node for StreamExpressionNode {
    fn accept_stream_input(&mut self, socket_type: &StreamSocketType, value: f32) {
        match socket_type {
            &StreamSocketType::Dynamic(uid) => {
                let local_index = self.values_in_mapping.iter().find(|mapping| mapping.0 == uid);

                if let Some(local_index) = local_index {
                    self.values_in[local_index.1] = value;
                }
            }
            _ => {}
        }
    }

    fn get_stream_output(&self, _socket_type: &StreamSocketType) -> f32 {
        self.value_out
    }

    fn process(&mut self, state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
        if let Some(ast) = &self.ast {
            // start by rewinding the scope
            self.scope.rewind(0);

            // add inputs to scope
            for (i, val) in self.values_in.iter().enumerate() {
                self.scope.push(format!("x{}", i + 1), *val);
            }

            // now we run the expression!
            let result = state.script_engine.eval_ast_with_scope::<f32>(&mut self.scope, &ast);

            // convert the output to a usuable form
            match result {
                Ok(output) => {
                    self.value_out = output;
                }
                Err(_) => {
                    self.value_out = 0.0;
                }
            }
        }

        NodeOk::no_warnings(())
    }

    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        let mut did_rows_change = false;
        let mut warnings = WarningBuilder::new();

        // these are the rows it always has
        let mut node_rows: Vec<NodeRow> = vec![
            NodeRow::Property(
                "expression".to_string(),
                PropertyType::String,
                Property::String("".to_string()),
            ),
            NodeRow::Property(
                "values_in_count".to_string(),
                PropertyType::Integer,
                Property::Integer(0),
            ),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0),
        ];

        let mut expression = "";
        if let Some(Property::String(new_expression)) = state.props.get("expression") {
            expression = new_expression;
        }

        if let Some(Property::Integer(values_in_count)) = state.props.get("values_in_count") {
            let values_in_count_usize = *values_in_count as usize;

            // is it bigger or smaller than last time?
            if values_in_count_usize > self.values_in.len() {
                // if bigger, add some accordingly
                for i in self.values_in.len()..values_in_count_usize {
                    // get ID for socket
                    let new_socket_uid = state
                        .registry
                        .register_socket(
                            format!("stream.stream_expression.{}", i),
                            SocketType::Stream(StreamSocketType::Audio),
                            "stream.stream_expression".to_string(),
                            Some(json! {{ "input_number": i + 1 }}),
                        )
                        .unwrap()
                        .1;

                    // add a socket -> local index mapping
                    self.values_in_mapping.push((new_socket_uid, i));
                    self.values_in.push(0.0);
                }

                did_rows_change = true;
            } else if values_in_count_usize < self.values_in.len() {
                // if smaller, see how many we need to remove
                let to_remove = self.values_in.len() - values_in_count_usize;

                for _ in 0..to_remove {
                    self.values_in.pop();
                    self.values_in_mapping.pop();
                }

                did_rows_change = true;
            }
            // if it's the same, we don't need to do anything
        } else {
            self.values_in.clear();
            self.values_in_mapping.clear();
        }

        for i in 0..self.values_in.len() {
            let new_socket_type = state
                .registry
                .register_socket(
                    format!("stream.stream_expression.{}", i),
                    SocketType::Stream(StreamSocketType::Audio),
                    "stream.stream_expression".to_string(),
                    Some(json! {{ "input_number": i + 1 }}),
                )
                .unwrap()
                .0
                .as_stream()
                .unwrap();

            node_rows.push(NodeRow::StreamInput(new_socket_type, 0.0));
        }

        if expression.is_empty() {
            // if it's empty, don't compile it
            self.ast = None;
        } else {
            // compile the expression and collect any errors
            let possible_ast = state.script_engine.compile(&expression);

            match possible_ast {
                Ok(ast) => {
                    self.ast = Some(ast);
                }
                Err(parser_error) => {
                    self.ast = None;
                    warnings.add_warning(NodeWarning::RhaiParserFailure { parser_error });
                }
            }
        }

        Ok(NodeOk::new(
            InitResult {
                did_rows_change,
                node_rows,
                changed_properties: None,
            },
            warnings.into_warnings(),
        ))
    }
}
