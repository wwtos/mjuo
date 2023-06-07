use std::mem;

use rhai::{Scope, AST};
use smallvec::SmallVec;

use crate::nodes::prelude::*;

use super::util::{value_to_dynamic, ProcessState};

#[derive(Debug, Clone)]
pub struct MidiFilterNode {
    filter: Option<AST>,
    filter_raw: String,
    midi_state: ProcessState<MidiBundle>,
    scope: Scope<'static>,
    output: Option<MidiBundle>,
}

impl NodeRuntime for MidiFilterNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let mut warning: Option<NodeWarning> = None;

        if let Some(Property::String(expression)) = state.props.get("expression") {
            let possible_ast = state.script_engine.compile(expression);
            self.filter_raw = expression.clone();

            match possible_ast {
                Ok(ast) => {
                    self.filter = Some(ast);
                }
                Err(err) => {
                    warning = Some(NodeWarning::RhaiParserFailure { parser_error: err });
                }
            }
        }

        InitResult::warning(warning)
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        _streams_in: &[&[f32]],
        _streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        let mut warning: Option<NodeWarning> = None;

        if let Some(filter) = &self.filter {
            self.midi_state = match &self.midi_state {
                ProcessState::Unprocessed(midi) => {
                    self.output = Some(
                        midi.iter()
                            .filter_map(|message| {
                                let midi_json = serde_json::to_value(&message.data).unwrap();

                                for (key, value) in midi_json.as_object().unwrap() {
                                    self.scope.push(key.as_str(), value_to_dynamic(value.clone()));
                                }

                                let result = state.script_engine.eval_ast_with_scope::<bool>(&mut self.scope, filter);

                                self.scope.rewind(0);

                                match result {
                                    Ok(output) => {
                                        if output {
                                            Some(message.clone())
                                        } else {
                                            None
                                        }
                                    }
                                    Err(err) => {
                                        warning = Some(NodeWarning::RhaiExecutionFailure {
                                            err: *err,
                                            script: self.filter_raw.clone(),
                                        });

                                        None
                                    }
                                }
                            })
                            .collect::<MidiBundle>(),
                    );

                    ProcessState::Processed
                }
                ProcessState::Processed => {
                    self.output = None;

                    ProcessState::None
                }
                ProcessState::None => ProcessState::None,
            };
        }

        ProcessResult::warning(warning)
    }

    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        self.midi_state = ProcessState::Unprocessed(midi_in[0].clone().unwrap());
    }

    fn get_midi_outputs(&mut self, midi_out: &mut [Option<MidiBundle>]) {
        midi_out[0] = mem::replace(&mut self.output, None);
    }
}

impl Node for MidiFilterNode {
    fn new(_sound_config: &SoundConfig) -> MidiFilterNode {
        MidiFilterNode {
            filter: None,
            filter_raw: "".into(),
            midi_state: ProcessState::None,
            scope: Scope::new(),
            output: None,
        }
    }

    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            midi_input(register("midi")),
            NodeRow::Property(
                "expression".to_string(),
                PropertyType::String,
                Property::String("".to_string()),
            ),
            midi_output(register("midi")),
        ])
    }
}
