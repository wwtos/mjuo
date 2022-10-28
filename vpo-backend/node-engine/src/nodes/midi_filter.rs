use rhai::{Scope, AST};
use sound_engine::midi::messages::MidiData;

use crate::{
    connection::MidiSocketType,
    errors::{ErrorsAndWarnings, FaultBuilder, NodeError, NodeOk, NodeResult},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
    property::{Property, PropertyType},
};

#[derive(Debug)]
enum MidiState {
    Unprocessed(Vec<MidiData>),
    Processed,
    None,
}

#[derive(Debug)]
pub struct MidiRouterNode {
    filter: Option<AST>,
    midi_state: MidiState,
    scope: Scope<'static>,
    output: Vec<MidiData>,
}

impl Node for MidiRouterNode {
    fn init(&mut self, state: NodeInitState) -> NodeResult<InitResult> {
        let mut fault_builder = FaultBuilder::default();

        if let Some(Property::String(expression)) = state.props.get("expression") {
            let possible_ast = state.script_engine.compile(expression);

            match possible_ast {
                Ok(ast) => {
                    self.filter = Some(ast);
                }
                Err(err) => {
                    fault_builder.add_error(NodeError::RhaiParserError(err));
                }
            }
        }

        InitResult::simple(vec![NodeRow::Property(
            "expression".to_string(),
            PropertyType::String,
            Property::String("".to_string()),
        )])
    }

    fn process(&mut self, state: NodeProcessState) -> NodeResult<()> {
        if let Some(filter) = &self.filter {
            self.midi_state = match &self.midi_state {
                MidiState::Unprocessed(midi) => {
                    self.output = midi
                        .iter()
                        .filter_map(|message| {
                            let rhai_midi = state
                                .script_engine
                                .parse_json(serde_json::to_string(&message).unwrap(), false)
                                .unwrap();

                            for (key, value) in rhai_midi.into_iter() {
                                self.scope.push(key.as_str(), value);
                            }

                            let result = state.script_engine.eval_ast_with_scope::<bool>(&mut self.scope, filter);

                            self.scope.rewind(0);

                            match result {
                                Ok(output) => {
                                    if output {
                                        Some(Ok(message.clone()))
                                    } else {
                                        None
                                    }
                                }
                                Err(err) => {
                                    // cleanup before erroring
                                    self.midi_state = MidiState::Processed;

                                    Some(Err(NodeError::RhaiEvalError(*err)))
                                }
                            }
                        })
                        .collect::<Result<Vec<MidiData>, NodeError>>()
                        .map_err(|err| ErrorsAndWarnings::err(err))?;

                    MidiState::Processed
                }
                MidiState::Processed => {
                    self.output = vec![];

                    MidiState::None
                }
                MidiState::None => MidiState::None,
            };
        }

        NodeOk::no_warnings(())
    }

    fn accept_midi_input(&mut self, _socket_type: &MidiSocketType, value: Vec<MidiData>) {
        self.midi_state = MidiState::Unprocessed(value);
    }

    fn get_midi_output(&self, _socket_type: &MidiSocketType) -> Vec<MidiData> {
        self.output.clone()
    }
}
