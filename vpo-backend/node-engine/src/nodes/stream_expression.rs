use rhai::{Scope, AST};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct StreamExpressionNode {
    ast: Option<Box<AST>>,
    scope: Box<Scope<'static>>,
}

impl NodeRuntime for StreamExpressionNode {
    fn process(
        &mut self,
        globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        _resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        if let Some(ast) = &self.ast {
            for (i, frame) in outs.streams[0].iter_mut().enumerate() {
                // start by rewinding the scope
                self.scope.rewind(0);

                // add inputs to scope
                for (j, val) in ins.streams.iter().enumerate() {
                    self.scope.push(format!("x{}", j + 1), val[i]);
                }

                // now we run the expression!
                let result = globals.script_engine.eval_ast_with_scope::<f32>(&mut self.scope, ast);

                // convert the output to a usuable form
                match result {
                    Ok(output) => {
                        *frame = output;
                    }
                    Err(_) => break,
                }
            }

            self.scope.rewind(0);
        }

        NodeOk::no_warnings(())
    }

    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let mut warning: Option<NodeWarning> = None;

        let expression = params
            .props
            .get("expression")
            .and_then(|x| x.clone().as_string())
            .unwrap_or("".into());

        if expression.is_empty() {
            // if it's empty, don't compile it
            self.ast = None;
        } else {
            // compile the expression and collect any errors
            let possible_ast = params.script_engine.compile(expression);

            match possible_ast {
                Ok(ast) => {
                    self.ast = Some(Box::new(ast));
                }
                Err(parser_error) => {
                    self.ast = None;

                    warning = Some(NodeWarning::RhaiParserFailure { parser_error });
                }
            }
        }

        InitResult::warning(warning)
    }
}

impl Node for StreamExpressionNode {
    fn new(_sound_config: &SoundConfig) -> StreamExpressionNode {
        StreamExpressionNode {
            scope: Box::new(Scope::new()),
            ast: None,
        }
    }

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
            stream_output(register("audio")),
        ];

        if let Some(Property::Integer(values_in_count)) = props.get("values_in_count") {
            for i in 0..(*values_in_count) {
                node_rows.push(NodeRow::Input(
                    Socket::Numbered(register("variable-numbered"), i + 1, SocketType::Value, 1),
                    SocketValue::Value(Primitive::Float(0.0)),
                ));
            }
        }

        NodeIo::simple(node_rows)
    }
}
