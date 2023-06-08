use std::mem;

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
    midi_in: Option<MidiBundle>,
    value_out: Option<Primitive>,
}

impl NodeRuntime for MidiToValueNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        if let Some(Property::String(expression)) = state.props.get("expression") {
            self.expression_raw = expression.clone();
        }

        // compile the expression and collect any errors
        let possible_ast = state.script_engine.compile(&self.expression_raw);

        match possible_ast {
            Ok(ast) => {
                self.ast = Some(Box::new(ast));

                InitResult::nothing()
            }
            Err(parser_error) => Ok(NodeOk {
                value: InitResult {
                    changed_properties: None,
                },
                warnings: vec![NodeWarning::RhaiParserFailure { parser_error }],
            }),
        }
    }

    fn accept_midi_inputs(&mut self, midi_in: &[Option<MidiBundle>]) {
        self.midi_in = midi_in[0].clone();
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        _streams_in: &[&[f32]],
        _streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        let mut warnings = vec![];

        if let (Some(midi_in), Some(ast)) = (&self.midi_in, self.ast.as_ref()) {
            for message in midi_in {
                self.scope.push("timestamp", message.timestamp);

                midi_to_scope(&mut self.scope, &message.data);

                let result = state.script_engine.eval_ast_with_scope::<Dynamic>(&mut self.scope, ast);

                match result {
                    Ok(dynamic) => {
                        self.value_out = dynamic_to_primitive(dynamic);
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

    fn get_value_outputs(&mut self, values_out: &mut [Option<Primitive>]) {
        values_out[0] = mem::replace(&mut self.value_out, None);
    }

    fn finish(&mut self) {
        self.midi_in = None;
    }
}

impl Node for MidiToValueNode {
    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            property("expression", PropertyType::String, Property::String("".into())),
            midi_input(register("midi")),
            value_output(register("value")),
        ])
    }

    fn new(_sound_config: &SoundConfig) -> Self {
        MidiToValueNode {
            ast: None,
            expression_raw: "".into(),
            scope: Box::new(Scope::new()),
            midi_in: None,
            value_out: None,
        }
    }
}
