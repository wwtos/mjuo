use std::collections::HashMap;

use rhai::{Dynamic, Engine, Scope, AST};
use serde_json::json;

use crate::connection::{Primitive, SocketType, ValueSocketType};
use crate::errors::{ErrorsAndWarnings, NodeError, NodeWarning};
use crate::node::{InitResult, Node, NodeRow};
use crate::node_graph::NodeGraph;
use crate::property::{Property, PropertyType};
use crate::socket_registry::SocketRegistry;
use crate::traversal::traverser::Traverser;

#[derive(Debug)]
pub struct ExpressionNode {
    ast: Option<AST>,
    scope: Scope<'static>,
    values_in: Vec<Primitive>,
    values_in_mapping: Vec<(u64, usize)>,
    value_out: Option<Primitive>,
    have_values_changed: bool,
}

impl ExpressionNode {
    pub fn new() -> ExpressionNode {
        ExpressionNode {
            scope: Scope::new(),
            ast: None,
            values_in: vec![],
            values_in_mapping: vec![],
            value_out: None,
            have_values_changed: true,
        }
    }
}

impl Node for ExpressionNode {
    fn accept_value_input(&mut self, socket_type: &ValueSocketType, value: Primitive) {
        match socket_type {
            &ValueSocketType::Dynamic(uid) => {
                let local_index = self.values_in_mapping.iter().find(|mapping| mapping.0 == uid);

                if let Some(local_index) = local_index {
                    self.values_in[local_index.1] = value;
                }

                self.have_values_changed = true;
            }
            _ => {}
        }
    }

    fn get_value_output(&self, _socket_type: &ValueSocketType) -> Option<Primitive> {
        self.value_out.clone()
    }

    fn process(
        &mut self,
        _current_time: i64,
        scripting_engine: &Engine,
        _inner_graph: Option<(&mut NodeGraph, &Traverser)>,
    ) -> Result<(), ErrorsAndWarnings> {
        if let Some(ast) = &self.ast {
            if self.have_values_changed {
                // add inputs to scope
                for (i, val) in self.values_in.iter().enumerate() {
                    self.scope.push(format!("x{}", i + 1), val.clone().as_dynamic());
                }

                // now we run the expression!
                let result = scripting_engine.eval_ast_with_scope::<Dynamic>(&mut self.scope, &ast);

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

                                return Err(ErrorsAndWarnings {
                                    errors: vec![],
                                    warnings: vec![NodeWarning::RhaiInvalidReturnType(output.type_name().to_string())],
                                });
                            }
                        }
                    }
                    Err(err) => {
                        // cleanup before erroring
                        self.have_values_changed = false;
                        self.scope.rewind(0);

                        return Err(ErrorsAndWarnings {
                            errors: vec![NodeError::RhaiEvalError(*err)],
                            warnings: vec![],
                        });
                    }
                }

                self.have_values_changed = false;
                self.scope.rewind(0);
            }
        }

        Ok(())
    }

    fn init(
        &mut self,
        properties: &HashMap<String, Property>,
        registry: &mut SocketRegistry,
        scripting_engine: &Engine,
    ) -> InitResult {
        let mut did_rows_change = false;

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
            NodeRow::ValueOutput(ValueSocketType::Default, Primitive::Float(0.0)),
        ];

        let mut expression = "";
        if let Some(Property::String(new_expression)) = properties.get("expression") {
            expression = new_expression;
        }

        if let Some(Property::Integer(values_in_count)) = properties.get("values_in_count") {
            let values_in_count_usize = *values_in_count as usize;

            // is it bigger or smaller than last time?
            if values_in_count_usize > self.values_in.len() {
                // if bigger, add some accordingly
                for i in self.values_in.len()..values_in_count_usize {
                    // get ID for socket
                    let new_socket_uid = registry
                        .register_socket(
                            format!("value.expression.{}", i),
                            SocketType::Value(ValueSocketType::Default),
                            "value.expression".to_string(),
                            Some(json! {{ "input_number": i + 1 }}),
                        )
                        .unwrap()
                        .1;

                    // add a socket -> local index mapping
                    self.values_in_mapping.push((new_socket_uid, i));
                    self.values_in.push(Primitive::Float(0.0));
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
            let new_socket_type = registry
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

            node_rows.push(NodeRow::ValueInput(new_socket_type, Primitive::Float(0.0)));
        }

        // compile the expression and collect any errors
        let possible_ast = scripting_engine.compile(&expression);
        let mut possible_error = None;

        match possible_ast {
            Ok(ast) => {
                self.ast = Some(ast);
            }
            Err(err) => {
                possible_error = Some(ErrorsAndWarnings {
                    errors: vec![NodeError::RhaiParserError(err)],
                    warnings: vec![],
                });
            }
        }

        self.have_values_changed = true;

        InitResult {
            did_rows_change,
            node_rows,
            changed_properties: None,
            errors_and_warnings: possible_error,
        }
    }
}
