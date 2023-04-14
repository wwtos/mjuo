use rhai::{Dynamic, Scope, AST};
use smallvec::SmallVec;

use crate::nodes::prelude::*;

use super::util::ProcessState;

#[derive(Debug, Clone)]
pub struct MidiFilterNode {
    filter: Option<AST>,
    filter_raw: String,
    midi_state: ProcessState<MidiBundle>,
    scope: Scope<'static>,
    output: Option<MidiBundle>,
}

impl MidiFilterNode {
    pub fn new() -> Self {
        MidiFilterNode {
            filter: None,
            filter_raw: "".into(),
            midi_state: ProcessState::None,
            scope: Scope::new(),
            output: None,
        }
    }
}

impl Default for MidiFilterNode {
    fn default() -> Self {
        Self::new()
    }
}

fn value_to_dynamic(value: serde_json::Value) -> Dynamic {
    match value {
        serde_json::Value::Null => Dynamic::from(()),
        serde_json::Value::Bool(value) => Dynamic::from(value),
        serde_json::Value::Number(value) => {
            if value.is_i64() {
                Dynamic::from(value.as_i64().unwrap() as i32)
            } else {
                Dynamic::from(value.as_f64().unwrap() as f32)
            }
        }
        serde_json::Value::String(value) => Dynamic::from(value),
        serde_json::Value::Array(array) => Dynamic::from(array.into_iter().map(value_to_dynamic)),
        serde_json::Value::Object(object) => Dynamic::from(
            object
                .into_iter()
                .map(|(k, v)| (smartstring::SmartString::from(k), value_to_dynamic(v)))
                .collect::<rhai::Map>(),
        ),
    }
}

impl NodeRuntime for MidiFilterNode {
    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let mut warnings = WarningBuilder::new();

        if let Some(Property::String(expression)) = state.props.get("expression") {
            let possible_ast = state.script_engine.compile(expression);
            self.filter_raw = expression.clone();

            match possible_ast {
                Ok(ast) => {
                    self.filter = Some(ast);
                }
                Err(err) => {
                    warnings.add_warning(NodeWarning::RhaiParserFailure { parser_error: err });
                }
            }
        }

        InitResult::nothing()
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        _streams_in: &[f32],
        _streams_out: &mut [f32],
    ) -> Result<NodeOk<()>, NodeError> {
        let mut warnings = WarningBuilder::new();

        if let Some(filter) = &self.filter {
            self.midi_state = match &self.midi_state {
                ProcessState::Unprocessed(midi) => {
                    self.output = Some(
                        midi.iter()
                            .filter_map(|message| {
                                let midi_json = serde_json::to_value(message).unwrap();

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
                                        warnings.add_warning(NodeWarning::RhaiExecutionFailure {
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

        Ok(NodeOk::new((), warnings.into_warnings()))
    }

    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        self.midi_state = ProcessState::Unprocessed(midi_in[0].clone().unwrap());
    }

    fn get_midi_outputs(&self, midi_out: &mut [Option<MidiBundle>]) {
        midi_out[0] = self.output.clone();
    }
}

impl Node for MidiFilterNode {
    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            midi_input(register("midi"), SmallVec::new()),
            NodeRow::Property(
                "expression".to_string(),
                PropertyType::String,
                Property::String("".to_string()),
            ),
            midi_output(register("midi"), SmallVec::new()),
        ])
    }
}
