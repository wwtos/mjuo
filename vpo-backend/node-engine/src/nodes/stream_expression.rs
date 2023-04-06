use rhai::{Scope, AST};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct StreamExpressionNode {
    ast: Option<Box<AST>>,
    scope: Box<Scope<'static>>,
}

impl Default for StreamExpressionNode {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamExpressionNode {
    pub fn new() -> StreamExpressionNode {
        StreamExpressionNode {
            scope: Box::new(Scope::new()),
            ast: None,
        }
    }
}

impl NodeRuntime for StreamExpressionNode {
    fn process(&mut self, state: NodeProcessState, streams_in: &[f32], streams_out: &mut [f32]) -> NodeResult<()> {
        if let Some(ast) = &self.ast {
            // start by rewinding the scope
            self.scope.rewind(0);

            // add inputs to scope
            for (i, val) in streams_in.iter().enumerate() {
                self.scope.push(format!("x{}", i + 1), *val);
            }

            // now we run the expression!
            let result = state.script_engine.eval_ast_with_scope::<f32>(&mut self.scope, ast);

            // convert the output to a usuable form
            match result {
                Ok(output) => {
                    streams_out[0] = output;
                }
                Err(_) => {}
            }
        }

        NodeOk::no_warnings(())
    }

    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let mut did_rows_change = false;
        let mut warnings = WarningBuilder::new();

        let expression = state
            .props
            .get("expression")
            .and_then(|x| x.clone().as_string())
            .unwrap_or("".into());

        let values_in_count = state
            .props
            .get("values_in_count")
            .and_then(|x| x.clone().as_integer())
            .unwrap() as usize;

        if expression.is_empty() {
            // if it's empty, don't compile it
            self.ast = None;
        } else {
            // compile the expression and collect any errors
            let possible_ast = state.script_engine.compile(expression);

            match possible_ast {
                Ok(ast) => {
                    self.ast = Some(Box::new(ast));
                }
                Err(parser_error) => {
                    self.ast = None;
                    warnings.add_warning(NodeWarning::RhaiParserFailure { parser_error });
                }
            }
        }

        InitResult::nothing()
    }
}

impl Node for StreamExpressionNode {
    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
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
            stream_output(register("audio"), 0.0),
        ];

        if let Some(Property::Integer(values_in_count)) = props.get("values_in_count") {
            for i in 0..(*values_in_count) {
                node_rows.push(NodeRow::Input(
                    Socket::Numbered(register("socket-variable-numbered"), i + 1, SocketType::Value, 1),
                    SocketValue::Value(Primitive::Float(0.0)),
                ));
            }
        }

        NodeIo::simple(node_rows)
    }
}
