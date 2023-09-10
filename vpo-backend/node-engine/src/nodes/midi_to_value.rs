use rhai::{Dynamic, Scope, AST};

use super::{
    prelude::*,
    util::{dynamic_to_primitive, midi_to_scope},
};

#[derive(Debug, Clone)]
pub struct MidiToValueNode {
    ast: Option<Box<AST>>,
    expression_raw: String,
    scope: Box<Scope<'static>>,
}

impl NodeRuntime for MidiToValueNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        if let Some(Property::String(expression)) = params.props.get("expression") {
            self.expression_raw = expression.clone();
        }

        // compile the expression and collect any errors
        let possible_ast = params.script_engine.compile(&self.expression_raw);

        match possible_ast {
            Ok(ast) => {
                self.ast = Some(Box::new(ast));

                InitResult::nothing()
            }
            Err(parser_error) => Ok(NodeOk {
                value: InitResult {
                    changed_properties: None,
                    needed_resources: vec![],
                },
                warnings: vec![NodeWarning::RhaiParserFailure { parser_error }],
            }),
        }
    }

    fn process(&mut self, context: NodeProcessContext, ins: Ins, outs: Outs, resources: &[&dyn Any]) -> NodeResult<()> {
        let mut warnings = vec![];

        if let Some(ast) = self.ast.as_ref() {
            for message in ins.midis[0][0] {
                self.scope.push("timestamp", message.timestamp);

                midi_to_scope(&mut self.scope, &message.data);

                let result = context
                    .script_engine
                    .eval_ast_with_scope::<Dynamic>(&mut self.scope, ast);

                match result {
                    Ok(dynamic) => {
                        outs.values[0][0] = dynamic_to_primitive(dynamic);
                    }
                    Err(err) => {
                        warnings.push(NodeWarning::RhaiExecutionFailure {
                            err: *err,
                            script: self.expression_raw.clone(),
                        });
                    }
                }

                self.scope.rewind(0);
            }
        }

        Ok(NodeOk { value: (), warnings })
    }
}

impl Node for MidiToValueNode {
    fn get_io(context: NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            property("expression", PropertyType::String, Property::String("".into())),
            midi_input("midi", 1),
            value_output("value", 1),
        ])
    }

    fn new(_sound_config: &SoundConfig) -> Self {
        MidiToValueNode {
            ast: None,
            expression_raw: "".into(),
            scope: Box::new(Scope::new()),
        }
    }
}
