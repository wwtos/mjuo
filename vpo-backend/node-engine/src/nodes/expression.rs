use std::cmp::Ordering;

use rhai::{Dynamic, Scope, AST};
use serde_json::json;

use crate::connection::{Primitive, SocketType, ValueSocketType};
use crate::errors::{NodeError, NodeOk, NodeWarning, WarningBuilder};
use crate::node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow};
use crate::property::{Property, PropertyType};

#[derive(Debug, Clone)]
pub struct ExpressionNode {
    ast: Option<Box<AST>>,
    scope: Box<Scope<'static>>,
    values_in: Vec<Primitive>,
    value_out: Option<Primitive>,
    have_values_changed: bool,
}

impl Default for ExpressionNode {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionNode {
    pub fn new() -> ExpressionNode {
        ExpressionNode {
            scope: Box::new(Scope::new()),
            ast: None,
            values_in: vec![],
            value_out: None,
            have_values_changed: true,
        }
    }
}

impl Node for ExpressionNode {
    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        self.have_values_changed = false;

        for (i, value_in) in values_in.iter().enumerate() {
            if let Some(value) = value_in {
                self.have_values_changed = true;
                self.values_in[i] = value.clone();
            }
        }
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        streams_in: &[f32],
        streams_out: &mut [f32],
    ) -> Result<NodeOk<()>, NodeError> {
        let mut warnings = WarningBuilder::new();

        self.value_out = None;
        if let Some(ast) = &self.ast {
            if self.have_values_changed {
                // add inputs to scope
                for (i, val) in self.values_in.iter().enumerate() {
                    self.scope.push(format!("x{}", i + 1), val.clone().as_dynamic());
                }

                // now we run the expression!
                let result = state.script_engine.eval_ast_with_scope::<Dynamic>(&mut self.scope, ast);

                // convert the output to a usuable form
                match result {
                    Ok(output) => {
                        self.value_out = match output.type_name() {
                            "bool" => Some(Primitive::Boolean(output.as_bool().unwrap())),
                            "string" => Some(Primitive::String(output.into_string().unwrap())),
                            "i32" => Some(Primitive::Int(output.as_int().unwrap())),
                            "f32" => Some(Primitive::Float(output.as_float().unwrap())),
                            _ => {
                                // cleanup before erroring
                                self.have_values_changed = false;
                                self.scope.rewind(0);

                                warnings.add_warning(NodeWarning::RhaiInvalidReturnType {
                                    return_type: output.type_name().to_string(),
                                });

                                None
                            }
                        }
                    }
                    Err(err) => {
                        // cleanup before erroring
                        self.have_values_changed = false;
                        self.scope.rewind(0);

                        return Err(NodeError::RhaiEvalError { result: *err });
                    }
                }

                self.scope.rewind(0);
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
            NodeRow::ValueOutput(ValueSocketType::Default, Primitive::Float(0.0), false),
        ];

        let mut expression = "";
        if let Some(Property::String(new_expression)) = state.props.get("expression") {
            expression = new_expression;
        }

        if let Some(Property::Integer(values_in_count)) = state.props.get("values_in_count") {
            let values_in_count = *values_in_count as usize;

            match values_in_count.cmp(&self.values_in.len()) {
                Ordering::Less => {
                    // if smaller, see how many we need to remove
                    let to_remove = self.values_in.len() - values_in_count;

                    for _ in 0..to_remove {
                        self.values_in.pop();
                    }

                    did_rows_change = true;
                }
                Ordering::Equal => {
                    // if it's the same, we don't need to do anything
                }
                Ordering::Greater => {
                    // if bigger, add some accordingly
                    for i in self.values_in.len()..values_in_count {
                        // get ID for socket
                        let new_socket_uid = state
                            .registry
                            .register_socket(
                                format!("value.expression.{}", i),
                                SocketType::Value(ValueSocketType::Default),
                                "value.expression".to_string(),
                                Some(json! {{ "input_number": i + 1 }}),
                            )
                            .unwrap()
                            .1;

                        // add a socket -> local index mapping
                        self.values_in.push(Primitive::Float(0.0));
                    }

                    did_rows_change = true;
                }
            }
        } else {
            self.values_in.clear();
        }

        for i in 0..self.values_in.len() {
            let new_socket_type = state
                .registry
                .register_socket(
                    format!("value.expression.{}", i),
                    SocketType::Value(ValueSocketType::Default),
                    "value.expression".to_string(),
                    Some(json! {{ "input_number": i + 1 }}),
                )
                .unwrap()
                .0
                .as_value()
                .unwrap();

            node_rows.push(NodeRow::ValueInput(new_socket_type, Primitive::Float(0.0), false));
        }

        // compile the expression and collect any errors
        let possible_ast = state.script_engine.compile(expression);

        match possible_ast {
            Ok(ast) => {
                self.ast = Some(Box::new(ast));
            }
            Err(parser_error) => {
                warnings.add_warning(NodeWarning::RhaiParserFailure { parser_error });
            }
        }

        self.have_values_changed = true;

        Ok(NodeOk::new(
            InitResult {
                did_rows_change,
                node_rows,
                changed_properties: None,
                child_graph_io: None,
            },
            warnings.into_warnings(),
        ))
    }
}
