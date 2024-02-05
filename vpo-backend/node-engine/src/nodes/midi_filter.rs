use rhai::{Scope, AST};

use crate::nodes::prelude::*;

use super::util::add_message_to_scope;

#[derive(Debug, Clone)]
pub struct MidiFilterNode {
    filter: Option<Box<AST>>,
    filter_raw: String,
    scope: Box<Scope<'static>>,
    scratch: Vec<bool>,
}

impl NodeRuntime for MidiFilterNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let mut warning: Option<NodeWarning> = None;

        if let Some(Property::String(expression)) = params.props.get("expression") {
            let possible_ast = params.script_engine.compile(expression);
            self.filter_raw = expression.clone();

            match possible_ast {
                Ok(ast) => {
                    self.filter = Some(Box::new(ast));
                }
                Err(err) => {
                    warning = Some(NodeWarning::RhaiParserFailure { parser_error: err });
                }
            }
        }

        InitResult::warning(warning)
    }

    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        midi_store: &mut MidiStore,
        _resources: &[Resource],
    ) {
        if let Some(filter) = &self.filter {
            if let Some(midi) = &ins.midi(0)[0] {
                let messages = midi_store.borrow_midi(midi).unwrap();

                self.scratch.clear();

                // create a list of trues and falses of which messages should be passed on
                let filtered = messages.iter().map(|msg| {
                    add_message_to_scope(&mut self.scope, &msg.data);

                    let result = context
                        .script_engine
                        .eval_ast_with_scope::<bool>(&mut self.scope, filter);

                    self.scope.rewind(0);

                    match result {
                        Ok(output) => output,
                        Err(_) => false,
                    }
                });

                self.scratch.extend(filtered);

                let new_len = self.scratch.iter().filter(|x| **x).count();
                let mut i = 0;

                let messages_out = midi_store.map_midis(midi, new_len, |messages, _| {
                    while self.scratch[i] == false {
                        i += 1;
                    }

                    let out = i;
                    i += 1;

                    messages[out].clone()
                });

                outs.midi(0)[0] = messages_out;
            }
        }
    }
}

impl Node for MidiFilterNode {
    fn new(_sound_config: &SoundConfig) -> MidiFilterNode {
        MidiFilterNode {
            filter: None,
            filter_raw: "".into(),
            scope: Box::new(Scope::new()),
            scratch: Vec::with_capacity(16),
        }
    }

    fn get_io(_context: NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            midi_input("midi", 1),
            NodeRow::Property(
                "expression".to_string(),
                PropertyType::String,
                Property::String("".to_string()),
            ),
            midi_output("midi", 1),
        ])
    }
}
