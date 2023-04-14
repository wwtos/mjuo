use rhai::{Dynamic, Scope, AST};

use crate::nodes::prelude::*;

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

impl NodeRuntime for ExpressionNode {
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

    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let mut did_rows_change = false;
        let mut warnings = WarningBuilder::new();

        let mut expression = "";
        if let Some(Property::String(new_expression)) = state.props.get("expression") {
            expression = new_expression;
        }

        if let Some(Property::Integer(values_in_count)) = state.props.get("values_in_count") {
            self.values_in.resize(*values_in_count as usize, Primitive::Float(0.0));
        } else {
            self.values_in.clear();
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

        InitResult::nothing()
    }
}

impl Node for ExpressionNode {
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
            value_output(register("default"), Primitive::Float(0.0)),
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
