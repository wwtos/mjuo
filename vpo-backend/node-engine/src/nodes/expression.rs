use rhai::{Dynamic, Scope, AST};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct ExpressionNode {
    ast: Option<Box<AST>>,
    scope: Box<Scope<'static>>,
    values_in: Vec<Primitive>,
}

impl NodeRuntime for ExpressionNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let mut warnings = Vec::new();

        let expression = params.props.get_string("expression")?;

        let values_in_count = params.props.get_int("values_in_count")?;
        self.values_in
            .resize(values_in_count.max(1) as usize, Primitive::Float(0.0));

        // compile the expression and collect any errors
        let possible_ast = params.script_engine.compile(expression);

        match possible_ast {
            Ok(ast) => {
                self.ast = Some(Box::new(ast));
            }
            Err(parser_error) => {
                warnings.push(NodeWarning::RhaiParserFailure { parser_error });
            }
        }

        InitResult::nothing()
    }

    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        _osc_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        let mut have_values_changed = false;

        for (i, value_in) in ins.values().enumerate() {
            if value_in[0].is_some() {
                have_values_changed = true;
                self.values_in[i] = value_in[0];
            }
        }

        if have_values_changed {
            if let Some(ast) = &self.ast {
                // add inputs to scope
                for (i, val) in self.values_in.iter().enumerate() {
                    self.scope.push(format!("x{}", i + 1), val.as_dynamic());
                }

                // now we run the expression!
                let result = context
                    .script_engine
                    .eval_ast_with_scope::<Dynamic>(&mut self.scope, ast);

                // convert the output to a usuable form
                match result {
                    Ok(output) => {
                        outs.value(0)[0] = match output.type_name() {
                            "bool" => bool(output.as_bool().unwrap()),
                            "i32" => int(output.as_int().unwrap()),
                            "f32" => float(output.as_float().unwrap()),
                            "()" => Primitive::None,
                            _ => {
                                // cleanup before erroring
                                self.scope.rewind(0);

                                Primitive::None
                            }
                        };
                    }
                    Err(_) => {
                        // cleanup before erroring
                        self.scope.rewind(0);

                        return;
                    }
                }

                self.scope.rewind(0);
            }
        }
    }
}

impl Node for ExpressionNode {
    fn new(_sound_config: &SoundConfig) -> ExpressionNode {
        ExpressionNode {
            scope: Box::new(Scope::new()),
            ast: None,
            values_in: vec![],
        }
    }

    fn get_io(_context: NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
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
            value_output("default", 1),
        ];

        if let Some(Property::Integer(values_in_count)) = props.get("values_in_count") {
            for i in 0..(*values_in_count) {
                node_rows.push(NodeRow::Input(
                    Socket::WithData("variable_numbered".into(), (i + 1).to_string(), SocketType::Value, 1),
                    SocketValue::Value(Primitive::Float(0.0)),
                ));
            }
        }

        NodeIo::simple(node_rows)
    }
}
