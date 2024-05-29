use std::{borrow::Cow, collections::BTreeMap};

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct FunctionNode {
    // traverser: BufferedTraverser,
    child_io_nodes: Option<(NodeIndex, NodeIndex)>,
}

impl NodeRuntime for FunctionNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let mut warning = None;

        if let Some(graph_and_io) = params.child_graph {
            // let NodeGraphAndIo {
            //     graph_index: _,
            //     input_index,
            //     output_index,
            // } = graph_and_io;

            // let (traverser, errors_and_warnings) = BufferedTraverser::new(
            //     graph_and_io.graph_index,
            //     params.graph_manager,
            //     params.script_engine,
            //     params.resources,
            //     params.current_time,
            //     params.sound_config.clone(),
            // )?;
            // self.traverser = traverser;

            // if errors_and_warnings.any() {
            //     warning = Some(NodeWarning::InternalErrorsAndWarnings { errors_and_warnings });
            // }

            // self.child_io_nodes = Some((input_index, output_index));
        }

        InitResult::warning(warning)
    }

    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a>,
        mut outs: Outs<'a>,
        osc_store: &mut OscStore,
        resources: &[Resource],
    ) {
        // let (child_input_node, child_output_node) = self.child_io_nodes.unwrap();

        // let subgraph_input_node = self.local_graph.get_node_mut(child_input_node).unwrap();
        // subgraph_input_node.accept_stream_input(StreamSocketType::Audio, streams_in[0]);

        // self.traverser
        //     .traverse(
        //         &mut self.local_graph,
        //         self.is_first_time,
        //         state.current_time,
        //         params.script_engine,
        //         state.global_state,
        //     )
        //     .map_err(|err| NodeError::InnerGraphErrors {
        //         errors_and_warnings: err,
        //     })?;

        // let subgraph_output_node = self.local_graph.get_node_mut(child_output_node).unwrap();
        // streams_out[0] = subgraph_output_node.get_stream_output(StreamSocketType::Audio);

        // self.is_first_time = false;
    }
}

impl Node for FunctionNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        FunctionNode {
            // traverser: BufferedTraverser::default(),
            child_io_nodes: None,
        }
    }

    fn get_io(context: NodeGetIoContext, props: SeaHashMap<String, Property>) -> NodeIo {
        let channels = default_channels(&props, context.default_channel_count);

        let mut internal_io: BTreeMap<(SocketType, SocketDirection, String), NodeIndex> = BTreeMap::new();

        if let Some(child_graph) = context.child_graph {
            for (index, node) in child_graph.nodes_data_iter() {
                // TODO: make this less dependant on InputsNode and OutputsNode
                if &node.get_node_type() == "InputsNode" || &node.get_node_type() == "OutputsNode" {
                    let name = node
                        .get_property("name")
                        .and_then(|x| x.as_string())
                        .unwrap_or("".to_owned());
                    let io_type = node
                        .get_property("type")
                        .and_then(|x| x.as_multiple_choice())
                        .unwrap_or("".to_owned());

                    let io_type = match io_type.as_str() {
                        "osc" => SocketType::Osc,
                        "value" => SocketType::Value,
                        "stream" => SocketType::Stream,
                        _ => SocketType::Stream,
                    };

                    match node.get_node_type().as_str() {
                        "InputsNode" => {
                            internal_io.insert((io_type, SocketDirection::Input, name), index);
                        }
                        "OutputsNode" => {
                            internal_io.insert((io_type, SocketDirection::Output, name), index);
                        }
                        _ => {}
                    }
                }
            }
        }

        NodeIo {
            node_rows: vec![
                with_channels(context.default_channel_count),
                stream_input("audio", channels),
                NodeRow::InnerGraph,
                stream_output("audio", channels),
            ],
            child_graph_io: Some(vec![
                (
                    Socket::Simple(Cow::Borrowed("audio"), SocketType::Stream, 1),
                    SocketDirection::Input,
                ),
                (
                    Socket::Simple(Cow::Borrowed("audio"), SocketType::Stream, 1),
                    SocketDirection::Output,
                ),
            ]),
        }
    }
}
