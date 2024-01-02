use std::borrow::Cow;

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
        midi_store: &mut MidiStoreInterface,
        resources: &[Resource],
    ) -> NodeResult<()> {
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

        NodeOk::no_warnings(())
    }
}

impl Node for FunctionNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        FunctionNode {
            // traverser: BufferedTraverser::default(),
            child_io_nodes: None,
        }
    }

    fn get_io(context: &NodeGetIoContext, props: HashMap<String, Property, BuildHasherDefault<SeaHasher>>) -> NodeIo {
        let polyphony = default_channels(&props, context.default_channel_count);

        NodeIo {
            node_rows: vec![
                with_channels(context.default_channel_count),
                stream_input("audio", polyphony),
                NodeRow::InnerGraph,
                stream_output("audio", polyphony),
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
