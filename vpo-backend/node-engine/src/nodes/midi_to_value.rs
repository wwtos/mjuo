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

    fn process(
        &mut self,
        globals: NodeProcessGlobals,
        ins: Ins,
        outs: Outs,
        _resources: &[Option<(ResourceIndex, &dyn Any)>],
    ) -> NodeResult<()> {
        let mut warnings = vec![];

        if let (Some(midi_in), Some(ast)) = (ins.midis[0], self.ast.as_ref()) {
            for message in midi_in {
                self.scope.push("timestamp", message.timestamp);

                midi_to_scope(&mut self.scope, &message.data);

                let result = globals
                    .script_engine
                    .eval_ast_with_scope::<Dynamic>(&mut self.scope, ast);

                match result {
                    Ok(dynamic) => {
                        outs.values[0] = dynamic_to_primitive(dynamic);
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
    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            property("expression", PropertyType::String, Property::String("".into())),
            midi_input("midi"),
            value_output("value"),
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
