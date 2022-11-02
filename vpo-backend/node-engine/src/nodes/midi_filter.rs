use rhai::{Dynamic, Scope, AST};
use sound_engine::midi::messages::MidiData;

use crate::{
    connection::MidiSocketType,
    errors::{NodeOk, NodeResult, NodeWarning, WarningBuilder},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
    property::{Property, PropertyType},
};

#[derive(Debug, Clone)]
enum MidiState {
    Unprocessed(Vec<MidiData>),
    Processed,
    None,
}

#[derive(Debug, Clone)]
pub struct MidiFilterNode {
    filter: Option<AST>,
    filter_raw: String,
    midi_state: MidiState,
    scope: Scope<'static>,
    output: Vec<MidiData>,
}

impl MidiFilterNode {
    pub fn new() -> Self {
        MidiFilterNode {
            filter: None,
            filter_raw: "".into(),
            midi_state: MidiState::None,
            scope: Scope::new(),
            output: Vec::new(),
        }
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
        serde_json::Value::Array(array) => Dynamic::from(array.into_iter().map(|x| value_to_dynamic(x))),
        serde_json::Value::Object(object) => Dynamic::from(
            object
                .into_iter()
                .map(|(k, v)| (smartstring::SmartString::from(k), value_to_dynamic(v)))
                .collect::<rhai::Map>(),
        ),
    }
}

impl Node for MidiFilterNode {
    fn init(&mut self, state: NodeInitState) -> NodeResult<InitResult> {
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

        InitResult::simple(vec![
            NodeRow::MidiInput(MidiSocketType::Default, Vec::new()),
            NodeRow::Property(
                "expression".to_string(),
                PropertyType::String,
                Property::String("".to_string()),
            ),
            NodeRow::MidiOutput(MidiSocketType::Default, Vec::new()),
        ])
    }

    fn process(&mut self, state: NodeProcessState) -> NodeResult<()> {
        let mut warnings = WarningBuilder::new();

        if let Some(filter) = &self.filter {
            self.midi_state = match &self.midi_state {
                MidiState::Unprocessed(midi) => {
                    self.output = midi
                        .iter()
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
                        .collect::<Vec<MidiData>>();

                    MidiState::Processed
                }
                MidiState::Processed => {
                    self.output = vec![];

                    MidiState::None
                }
                MidiState::None => MidiState::None,
            };
        }

        Ok(NodeOk::new((), warnings.into_warnings()))
    }

    fn accept_midi_input(&mut self, _socket_type: &MidiSocketType, value: Vec<MidiData>) {
        self.midi_state = MidiState::Unprocessed(value);
    }

    fn get_midi_output(&self, _socket_type: &MidiSocketType) -> Vec<MidiData> {
        self.output.clone()
    }
}
