use rhai::{Scope, AST};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct StreamExpressionNode {
    ast: Option<Box<AST>>,
    scope: Box<Scope<'static>>,
}

impl NodeRuntime for StreamExpressionNode {
    fn process(&mut self, context: NodeProcessContext, ins: Ins, outs: Outs, resources: &[&dyn Any]) -> NodeResult<()> {
        if let Some(ast) = &self.ast {
            for (channel_i, channel_out) in outs.streams[0].iter_mut().enumerate() {
                for (frame_i, frame_out) in channel_out.iter_mut().enumerate() {
                    // start by rewinding the scope
                    self.scope.rewind(0);

                    // add inputs to scope
                    for (j, val) in ins.streams.iter().enumerate() {
                        self.scope.push(format!("x{}", j + 1), val[frame_i][channel_i]);
                    }

                    // now we run the expression!
                    let result = context.script_engine.eval_ast_with_scope::<f32>(&mut self.scope, ast);

                    // convert the output to a usuable form
                    match result {
                        Ok(output) => {
                            *frame_out = output;
                        }
                        Err(_) => break,
                    }
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

    fn get_io(context: NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        let channels = default_channels(&props, context.default_channel_count);

        // these are the rows it always has
        let mut node_rows: Vec<NodeRow> = vec![
            with_channels(context.default_channel_count),
            property("expression", PropertyType::String, Property::String("".into())),
            property("values_in_count", PropertyType::Integer, Property::Integer(0)),
            stream_output("audio", channels),
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
