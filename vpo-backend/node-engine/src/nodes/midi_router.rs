use std::collections::HashMap;

use rhai::{Engine, ParseError, Scope, AST};
use serde_json::json;
use sound_engine::midi::messages::{Channel, MidiData, Note, Velocity};

use crate::{
    connection::{SocketType, ValueSocketType},
    errors::{
        ErrorsAndWarnings,
        NodeError::{self, RhaiParserError},
    },
    node::{InitResult, Node, NodeRow},
    property::{Property, PropertyType},
    socket_registry::SocketRegistry,
};

enum MidiState {
    Unprocessed(Vec<MidiData>),
    Processed,
    None,
}

#[derive(Debug)]
struct MidiRouterRule {
    scope: Scope<'static>,
    filter_ast: Option<AST>,
    unparsed_code: String,
}

impl Default for MidiRouterRule {
    fn default() -> Self {
        Self {
            scope: Scope::new(),
            filter_ast: None,
            unparsed_code: String::default(),
        }
    }
}

#[derive(Debug)]
pub struct MidiRouterNode {
    rules: Vec<MidiRouterRule>,
}

impl Node for MidiRouterNode {
    fn init(
        &mut self,
        props: &HashMap<String, Property>,
        registry: &mut SocketRegistry,
        scripting_engine: &Engine,
    ) -> InitResult {
        // these are the rows it always has
        let mut node_rows: Vec<NodeRow> = vec![NodeRow::Property(
            "rules_count".to_string(),
            PropertyType::Integer,
            Property::Integer(0),
        )];

        // apply all the incoming field changes
        let errors = self
            .rules
            .iter_mut()
            .enumerate()
            .map(|(i, rule)| {
                if let Some(filter_unparsed) = props.get(&format!("filter_{}", i)) {
                    rule.unparsed_code = filter_unparsed.clone().as_string().unwrap();

                    let possible_ast = scripting_engine.compile(&rule.unparsed_code);

                    match possible_ast {
                        Ok(ast) => {
                            rule.filter_ast = Some(ast);
                            Ok(())
                        }
                        Err(err) => Err(NodeError::RhaiParserError(err)),
                    }
                } else {
                    Ok(())
                }
            })
            .filter_map(|possible_error| possible_error.err())
            .collect::<Vec<NodeError>>();

        if !errors.is_empty() {
            return InitResult {
                did_rows_change: false,
                node_rows,
                changed_properties: None,
                errors_and_warnings: Some(ErrorsAndWarnings {
                    errors,
                    warnings: vec![],
                }),
            };
        }

        if let Some(Property::Integer(rules_count)) = props.get("rules_count") {
            self.rules.resize_with(*rules_count as usize, MidiRouterRule::default);
        }

        // figure out what additional rows we need based on the rules
        let additional_rows = self
            .rules
            .iter()
            .enumerate()
            .map(|(i, rule)| {
                Ok::<[NodeRow; 2], NodeError>([
                    NodeRow::Property(
                        format!("filter_{}", i),
                        PropertyType::String,
                        Property::String(rule.unparsed_code.clone()),
                    ),
                    NodeRow::MidiOutput(
                        registry
                            .register_socket(
                                format!("value.midi_router.{}", i),
                                SocketType::Value(ValueSocketType::Default),
                                "value.midi_router".to_string(),
                                Some(json! {{ "route_number": i + 1 }}),
                            )?
                            .0
                            .as_midi()
                            .unwrap(),
                        vec![],
                    ),
                ])
            })
            .collect::<Result<Vec<[NodeRow; 2]>, NodeError>>();

        let additional_rows = match additional_rows {
            Ok(rows) => todo!(),
            Err(err) => todo!(),
        }

        InitResult::simple(node_rows)
    }
}
