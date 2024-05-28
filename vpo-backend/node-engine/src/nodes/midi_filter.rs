use rhai::{Scope, AST};

use crate::nodes::prelude::*;

use super::util::add_message_to_scope;

#[derive(Debug, Clone)]
pub struct MidiFilterNode {
    filter: Option<Box<AST>>,
    filter_raw: String,
    scope: Box<Scope<'static>>,
    scratch: Vec<u8>,
}

impl NodeRuntime for MidiFilterNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let mut warning: Option<NodeWarning> = None;

        let expression = params.props.get_string("expression")?;

        let possible_ast = params.script_engine.compile(&expression);
        self.filter_raw = expression;

        match possible_ast {
            Ok(ast) => {
                self.filter = Some(Box::new(ast));
            }
            Err(err) => {
                warning = Some(NodeWarning::RhaiParserFailure { parser_error: err });
            }
        }

        InitResult::warning(warning)
    }

    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        osc_store: &mut OscStore,
        _resources: &[Resource],
    ) {
        self.scratch.clear();

        let Some(filter) = &self.filter else { return };

        let Some(view) = ins.osc(0)[0]
            .get_messages(osc_store)
            .and_then(|bytes| OscView::new(bytes))
        else {
            return;
        };

        view.all_messages(|_, _, msg| {
            println!("message into filter: {:?}", msg);

            add_message_to_scope(&mut self.scope, msg);

            let result = context
                .script_engine
                .eval_ast_with_scope::<bool>(&mut self.scope, filter);

            self.scope.rewind(0);

            let should_be_passed = match result {
                Ok(output) => output,
                Err(_) => false,
            };

            if should_be_passed {
                write_message(&mut self.scratch, msg);
            }
        });

        outs.osc(0)[0] = write_bundle_and_message_scratch(osc_store, &self.scratch);
    }
}

impl Node for MidiFilterNode {
    fn new(_sound_config: &SoundConfig) -> MidiFilterNode {
        MidiFilterNode {
            filter: None,
            filter_raw: "".into(),
            scope: Box::new(Scope::new()),
            scratch: Vec::with_capacity(64),
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
