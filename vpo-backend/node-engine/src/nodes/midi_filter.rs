use rhai::{Scope, AST};

use crate::nodes::prelude::*;

use super::util::midi_to_scope;

#[derive(Debug, Clone)]
pub struct MidiFilterNode {
    filter: Option<Box<AST>>,
    filter_raw: String,
    scope: Box<Scope<'static>>,
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

    fn process<'brand>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'_, 'brand>,
        outs: Outs<'_, 'brand>,
        token: &mut GhostToken<'brand>,
        resources: &[&dyn Any],
    ) -> NodeResult<()> {
        let mut warning: Option<NodeWarning> = None;

        if let Some(filter) = &self.filter {
            if !ins.midis[0][0].borrow(token).is_empty() {
                let midi = &ins.midis[0][0].borrow(token);

                let output = midi
                    .iter()
                    .filter_map(|message| {
                        midi_to_scope(&mut self.scope, &message.data);

                        let result = context
                            .script_engine
                            .eval_ast_with_scope::<bool>(&mut self.scope, filter);

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
                    .collect::<MidiBundle>();

                *outs.midis[0][0].borrow_mut(token) = output;
            }
        }

        ProcessResult::warning(warning)
    }
}

impl Node for MidiFilterNode {
    fn new(_sound_config: &SoundConfig) -> MidiFilterNode {
        MidiFilterNode {
            filter: None,
            filter_raw: "".into(),
            scope: Box::new(Scope::new()),
        }
    }

    fn get_io(context: NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
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
