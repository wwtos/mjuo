use rhai::{Dynamic, Scope, AST};

use super::{
    prelude::*,
    util::{add_message_to_scope, dynamic_to_primitive},
};

#[derive(Debug, Clone)]
pub struct MidiToValueNode {
    ast: Option<Box<AST>>,
    expression_raw: String,
    scope: Box<Scope<'static>>,
}

impl NodeRuntime for MidiToValueNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let expression = params.props.get_string("expression")?;
        self.expression_raw = expression.clone();

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

    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        midi_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        let Some(ast) = self.ast.as_ref() else { return };
        let Some(midi) = &ins.midi(0)[0] else { return };

        let messages = midi_store.borrow_osc(midi).unwrap();

        for message in messages.iter() {
            self.scope.push("timestamp", message.timestamp);

            add_message_to_scope(&mut self.scope, &message.data);

            let result = context
                .script_engine
                .eval_ast_with_scope::<Dynamic>(&mut self.scope, ast);

            match result {
                Ok(dynamic) => {
                    outs.value(0)[0] = dynamic_to_primitive(dynamic);
                }
                Err(_) => {}
            }

            self.scope.rewind(0);
        }
    }
}

impl Node for MidiToValueNode {
    fn get_io(_context: NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
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
