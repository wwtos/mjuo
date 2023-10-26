use std::{cell::Cell, collections::BTreeMap, fmt::Debug, ops::Range};

use ghost_cell::{GhostCell, GhostToken};
use resource_manager::{ResourceId, ResourceIndex};
use rhai::Engine;
use smallvec::SmallVec;
use sound_engine::{MidiBundle, SoundConfig};

use crate::{
    connection::{Primitive, Socket, SocketType},
    errors::{ErrorsAndWarnings, NodeError, NodeWarning},
    global_state::{ResourceType, Resources},
    graph_manager::{GraphIndex, GraphManager},
    node::{Ins, NodeIndex, NodeInitParams, NodeRow, NodeRuntime, NodeState, Outs},
    node_graph::NodeGraph,
    nodes::{new_variant, NodeVariant},
};

struct NodeContainer<'a, 'arena, 'brand> {
    node: NodeVariant,
    to_input: Vec<(usize, Vec<Primitive>)>,
    ins: Ins<'a, 'arena, 'brand>,
    outs: Outs<'a, 'arena, 'brand>,
}

#[derive(Debug, Clone, Default)]
pub struct BufferedTraverser {}

impl BufferedTraverser {
    pub fn new(
        graph_index: GraphIndex,
        graph_manager: &GraphManager,
        script_engine: &Engine,
        resources: &Resources,
        current_time: i64,
        sound_config: SoundConfig,
    ) -> Result<(BufferedTraverser, ErrorsAndWarnings), NodeError> {
        let mut traverser = BufferedTraverser::default();

        traverser
            .init_graph(
                graph_index,
                graph_manager,
                script_engine,
                resources,
                current_time,
                sound_config,
            )
            .map(|errors_and_warnings| (traverser, errors_and_warnings))
    }

    pub fn init_graph(
        &mut self,
        graph_index: GraphIndex,
        graph_manager: &GraphManager,
        script_engine: &Engine,
        resources: &Resources,
        current_time: i64,
        sound_config: SoundConfig,
    ) -> Result<ErrorsAndWarnings, NodeError> {
        Ok(ErrorsAndWarnings::default())
    }
}

struct Indexes {
    pub streams: Vec<Vec<Range<usize>>>,
    pub midis: Vec<Range<usize>>,
    pub values: Vec<Range<usize>>,
    pub max_stream_channels: usize,
    pub max_midi_channels: usize,
    pub max_value_channels: usize,
}

// pub fn calc_indexes(
//     layout: &Layout,
//     graph_index: GraphIndex,
//     graph_manager: &GraphManager,
// ) -> Result<Indexes, NodeError> {
//     let Layout {
//         nodes,
//         resources_tracking,
//         nodes_linked_to_ui,
//         traversal_order,
//     } = layout;

//     let graph = graph_manager.get_graph(graph_index)?;

//     let stream_count = nodes.iter().map(|(_, node)| node.stream_outputs.iter().count()).count();
//     let midi_count = nodes.iter().map(|(_, node)| node.midi_outputs.iter().count()).count();
//     let value_count = nodes.iter().map(|(_, node)| node.value_outputs.iter().count()).count();

//     let max_stream_channels = nodes
//         .iter()
//         .map(|(_, node)| node.stream_outputs.iter().map(Socket::channels).max().unwrap_or(1))
//         .max()
//         .unwrap_or(1);
//     let max_midi_channels = nodes
//         .iter()
//         .map(|(_, node)| node.midi_outputs.iter().map(Socket::channels).max().unwrap_or(1))
//         .max()
//         .unwrap_or(1);
//     let max_value_channels = nodes
//         .iter()
//         .map(|(_, node)| node.value_outputs.iter().map(Socket::channels).max().unwrap_or(1))
//         .max()
//         .unwrap_or(1);

//     let mut streams = vec![0.0; stream_count * buffer_size];
//     let mut midis = vec![MidiBundle::new(); midi_count];
//     let mut values = vec![Primitive::None; value_count];

//     let mut stream_io: Vec<Vec<Range<usize>>> = vec![];
//     let mut midi_io: Vec<&[GhostCell<MidiBundle>]> = vec![];
//     let mut value_io: Vec<&[GhostCell<Primitive>]> = vec![];

//     let mut containers: Vec<NodeContainer> = vec![];

//     // # Step 2, populate mappings between nodes
//     // Now we know where all the nodes are, so we can tell the each node where it can
//     // find its input from
//     for index in traversal_order {
//         let instance = graph.get_node(*index).expect("node to exist");

//         let input_sockets = instance.list_input_sockets();
//         let output_sockets = instance.list_output_sockets();

//         let stream_io_input_index = stream_io.len();
//         let midi_io_input_index = midi_io.len();
//         let value_io_input_index = value_io.len();

//         let mut stream_io_inputs = 0;
//         let mut midi_io_inputs = 0;
//         let mut value_io_inputs = 0;

//         let my_layout = nodes.get(index).unwrap();

//         // let's look through this node's inputs
//         for input in &input_sockets {
//             // is this node's input socket connected to anything?
//             if let Some(connection_index) = graph.get_input_connection_index(*index, input).unwrap() {
//                 // get the node that it's connected from
//                 let connection = graph.get_graph().get_edge(connection_index.0).expect("edge to exist");
//                 let from_index = NodeIndex(connection.get_from());

//                 let from = graph.get_node(from_index).unwrap();
//                 // ensure same channel length
//                 assert!(from.list_output_sockets().iter().any(|socket| socket == input));

//                 // where is the other nodes' output location?
//                 let from_layout = nodes.get(&from_index).unwrap();

//                 // add it to the mapping
//                 match input.socket_type() {
//                     SocketType::Stream => {
//                         let position_in_stream = from_layout
//                             .stream_outputs
//                             .iter()
//                             .position(|other_socket| other_socket == &connection.data.from_socket)
//                             .unwrap()
//                             + from_layout.stream_index;

//                         // stream_io.push()

//                         stream_io.push(&channels[position_in_stream..(position_in_stream + input.channels())]);
//                         stream_io_inputs += 1;
//                     }
//                     SocketType::Midi => {
//                         let position_in_midi = from_layout
//                             .midi_outputs
//                             .iter()
//                             .position(|other_socket| other_socket == &connection.data.from_socket)
//                             .unwrap()
//                             + from_layout.midi_index;

//                         midi_io.push(&midis[position_in_midi..(position_in_midi + input.channels())]);
//                         midi_io_inputs += 1;
//                     }
//                     SocketType::Value => {
//                         let position_in_value = from_layout
//                             .value_outputs
//                             .iter()
//                             .position(|other_socket| other_socket == &connection.data.from_socket)
//                             .unwrap()
//                             + from_layout.value_index;

//                         value_io.push(&values[position_in_value..(position_in_value + input.channels())]);
//                         value_io_inputs += 1;
//                     }
//                     SocketType::NodeRef => {}
//                 }
//             } else {
//                 // it's not connected to anything, so push in a slice to nothing
//                 // (to preserve alignment)
//                 match input.socket_type() {
//                     SocketType::Stream => {
//                         stream_io.push(&dangling_channels[0..input.channels()]);
//                         stream_io_inputs += 1;
//                     }
//                     SocketType::Midi => {
//                         midi_io.push(&dangling_midi[0..input.channels()]);
//                         midi_io_inputs += 1;
//                     }
//                     SocketType::Value => {
//                         value_io.push(&dangling_value[0..input.channels()]);
//                         value_io_inputs += 1;
//                     }
//                     SocketType::NodeRef => {}
//                 }
//             }
//         }

//         let stream_io_output_index = stream_io.len();
//         let midi_io_output_index = midi_io.len();
//         let value_io_output_index = value_io.len();

//         let mut stream_io_outputs = 0;
//         let mut midi_io_outputs = 0;
//         let mut value_io_outputs = 0;

//         for output in &output_sockets {
//             // add outputs to the mapping
//             match output.socket_type() {
//                 SocketType::Stream => {
//                     let position_in_stream = my_layout
//                         .stream_outputs
//                         .iter()
//                         .position(|other_socket| other_socket == *output)
//                         .unwrap()
//                         + my_layout.stream_index;

//                     stream_io.push(&channels[position_in_stream..(position_in_stream + output.channels())]);
//                     stream_io_outputs += 1;
//                 }
//                 SocketType::Midi => {
//                     let position_in_midi = my_layout
//                         .midi_outputs
//                         .iter()
//                         .position(|other_socket| other_socket == *output)
//                         .unwrap()
//                         + my_layout.midi_index;

//                     midi_io.push(&midis[position_in_midi..(position_in_midi + output.channels())]);
//                     midi_io_outputs += 1;
//                 }
//                 SocketType::Value => {
//                     let position_in_value = my_layout
//                         .value_outputs
//                         .iter()
//                         .position(|other_socket| other_socket == *output)
//                         .unwrap()
//                         + my_layout.value_index;

//                     value_io.push(&values[position_in_value..(position_in_value + output.channels())]);
//                     value_io_outputs += 1;
//                 }
//                 SocketType::NodeRef => {}
//             }
//         }

//         // finally, finally, finally!!!
//         let layout = nodes.remove(index).unwrap();

//         containers.push(NodeContainer {
//             node: layout.node,
//             to_input: layout.to_input,
//             ins: Ins {
//                 midis: &midi_io[midi_io_input_index..(midi_io_input_index + midi_io_inputs)],
//                 values: &value_io[value_io_input_index..(value_io_input_index + value_io_inputs)],
//                 streams: &stream_io[stream_io_input_index..(stream_io_input_index + stream_io_inputs)],
//             },
//             outs: Outs {
//                 midis: &midi_io[midi_io_output_index..(midi_io_output_index + midi_io_outputs)],
//                 values: &value_io[value_io_output_index..(value_io_output_index + value_io_outputs)],
//                 streams: &stream_io[stream_io_output_index..(stream_io_output_index + stream_io_outputs)],
//             },
//         });
//     }

//     ()
// }

// pub fn create_traverser(
//     graph_index: GraphIndex,
//     graph_manager: &GraphManager,
//     script_engine: &Engine,
//     resources: &Resources,
//     current_time: i64,
//     sound_config: SoundConfig,
//     default_channel_count: usize,
// ) -> Result<(), NodeError> {
//     let buffer_size = sound_config.buffer_size;

//     let graph = graph_manager.get_graph(graph_index)?;

//     let Layout {
//         mut nodes,
//         resources_tracking,
//         nodes_linked_to_ui,
//         traversal_order,
//     } = layout_and_init_nodes(
//         &graph,
//         BTreeMap::new(), // TODO
//         &sound_config,
//         script_engine,
//         resources,
//         current_time,
//         graph_manager,
//         default_channel_count,
//     )?;

//     let stream_count = nodes.iter().map(|(_, node)| node.stream_outputs.iter().count()).count();
//     let midi_count = nodes.iter().map(|(_, node)| node.midi_outputs.iter().count()).count();
//     let value_count = nodes.iter().map(|(_, node)| node.value_outputs.iter().count()).count();

//     let max_stream_channels = nodes
//         .iter()
//         .map(|(_, node)| node.stream_outputs.iter().map(Socket::channels).max().unwrap_or(1))
//         .max()
//         .unwrap_or(1);
//     let max_midi_channels = nodes
//         .iter()
//         .map(|(_, node)| node.midi_outputs.iter().map(Socket::channels).max().unwrap_or(1))
//         .max()
//         .unwrap_or(1);
//     let max_value_channels = nodes
//         .iter()
//         .map(|(_, node)| node.value_outputs.iter().map(Socket::channels).max().unwrap_or(1))
//         .max()
//         .unwrap_or(1);

//     let mut dangling_stream: Vec<f32> = vec![0.0; buffer_size * max_stream_channels];
//     let mut dangling_midi = vec![MidiBundle::new(); max_midi_channels];
//     let mut dangling_value = vec![Primitive::None; max_value_channels];

//     // the mother of them all
//     let mut streams = vec![0.0; stream_count * buffer_size];
//     let mut midis = vec![MidiBundle::new(); midi_count];
//     let mut values = vec![Primitive::None; value_count];

//     GhostToken::new(|mut token| {
//         let streams = GhostCell::from_mut(&mut streams[..]).as_slice_of_cells();
//         let midis = GhostCell::from_mut(&mut midis[..]).as_slice_of_cells();
//         let values = GhostCell::from_mut(&mut values[..]).as_slice_of_cells();

//         let dangling_stream = GhostCell::from_mut(&mut dangling_stream[..]).as_slice_of_cells();
//         let dangling_midi = GhostCell::from_mut(&mut dangling_midi[..]).as_slice_of_cells();
//         let dangling_value = GhostCell::from_mut(&mut dangling_value[..]).as_slice_of_cells();

//         let channels: Vec<&[GhostCell<f32>]> = (0..stream_count)
//             .map(|channel_number| &streams[(channel_number * buffer_size)..((channel_number + 1) * buffer_size)])
//             .collect();

//         let dangling_channels: Vec<&[GhostCell<f32>]> = (0..max_stream_channels)
//             .map(|i| &dangling_stream[(i * buffer_size)..((i + 1) * buffer_size)])
//             .collect();

//         let mut stream_io: Vec<&[&[GhostCell<f32>]]> = vec![];
//         let mut midi_io: Vec<&[GhostCell<MidiBundle>]> = vec![];
//         let mut value_io: Vec<&[GhostCell<Primitive>]> = vec![];

//         let mut containers: Vec<NodeContainer> = vec![];

//         // # Step 2, populate mappings between nodes
//         // Now we know where all the nodes are, so we can tell the each node where it can
//         // find its input from
//         for index in &traversal_order {
//             let instance = graph.get_node(*index).expect("node to exist");

//             let input_sockets = instance.list_input_sockets();
//             let output_sockets = instance.list_output_sockets();

//             let stream_io_input_index = stream_io.len();
//             let midi_io_input_index = midi_io.len();
//             let value_io_input_index = value_io.len();

//             let mut stream_io_inputs = 0;
//             let mut midi_io_inputs = 0;
//             let mut value_io_inputs = 0;

//             let my_layout = nodes.get(index).unwrap();

//             // let's look through this node's inputs
//             for input in &input_sockets {
//                 // is this node's input socket connected to anything?
//                 if let Some(connection_index) = graph.get_input_connection_index(*index, input).unwrap() {
//                     // get the node that it's connected from
//                     let connection = graph.get_graph().get_edge(connection_index.0).expect("edge to exist");
//                     let from_index = NodeIndex(connection.get_from());

//                     let from = graph.get_node(from_index).unwrap();
//                     // ensure same channel length
//                     assert!(from.list_output_sockets().iter().any(|socket| socket == input));

//                     // where is the other nodes' output location?
//                     let from_layout = nodes.get(&from_index).unwrap();

//                     // add it to the mapping
//                     match input.socket_type() {
//                         SocketType::Stream => {
//                             let position_in_stream = from_layout
//                                 .stream_outputs
//                                 .iter()
//                                 .position(|other_socket| other_socket == &connection.data.from_socket)
//                                 .unwrap()
//                                 + from_layout.stream_index;

//                             stream_io.push(&channels[position_in_stream..(position_in_stream + input.channels())]);
//                             stream_io_inputs += 1;
//                         }
//                         SocketType::Midi => {
//                             let position_in_midi = from_layout
//                                 .midi_outputs
//                                 .iter()
//                                 .position(|other_socket| other_socket == &connection.data.from_socket)
//                                 .unwrap()
//                                 + from_layout.midi_index;

//                             midi_io.push(&midis[position_in_midi..(position_in_midi + input.channels())]);
//                             midi_io_inputs += 1;
//                         }
//                         SocketType::Value => {
//                             let position_in_value = from_layout
//                                 .value_outputs
//                                 .iter()
//                                 .position(|other_socket| other_socket == &connection.data.from_socket)
//                                 .unwrap()
//                                 + from_layout.value_index;

//                             value_io.push(&values[position_in_value..(position_in_value + input.channels())]);
//                             value_io_inputs += 1;
//                         }
//                         SocketType::NodeRef => {}
//                     }
//                 } else {
//                     // it's not connected to anything, so push in a slice to nothing
//                     // (to preserve alignment)
//                     match input.socket_type() {
//                         SocketType::Stream => {
//                             stream_io.push(&dangling_channels[0..input.channels()]);
//                             stream_io_inputs += 1;
//                         }
//                         SocketType::Midi => {
//                             midi_io.push(&dangling_midi[0..input.channels()]);
//                             midi_io_inputs += 1;
//                         }
//                         SocketType::Value => {
//                             value_io.push(&dangling_value[0..input.channels()]);
//                             value_io_inputs += 1;
//                         }
//                         SocketType::NodeRef => {}
//                     }
//                 }
//             }

//             let stream_io_output_index = stream_io.len();
//             let midi_io_output_index = midi_io.len();
//             let value_io_output_index = value_io.len();

//             let mut stream_io_outputs = 0;
//             let mut midi_io_outputs = 0;
//             let mut value_io_outputs = 0;

//             for output in &output_sockets {
//                 // add outputs to the mapping
//                 match output.socket_type() {
//                     SocketType::Stream => {
//                         let position_in_stream = my_layout
//                             .stream_outputs
//                             .iter()
//                             .position(|other_socket| other_socket == *output)
//                             .unwrap()
//                             + my_layout.stream_index;

//                         stream_io.push(&channels[position_in_stream..(position_in_stream + output.channels())]);
//                         stream_io_outputs += 1;
//                     }
//                     SocketType::Midi => {
//                         let position_in_midi = my_layout
//                             .midi_outputs
//                             .iter()
//                             .position(|other_socket| other_socket == *output)
//                             .unwrap()
//                             + my_layout.midi_index;

//                         midi_io.push(&midis[position_in_midi..(position_in_midi + output.channels())]);
//                         midi_io_outputs += 1;
//                     }
//                     SocketType::Value => {
//                         let position_in_value = my_layout
//                             .value_outputs
//                             .iter()
//                             .position(|other_socket| other_socket == *output)
//                             .unwrap()
//                             + my_layout.value_index;

//                         value_io.push(&values[position_in_value..(position_in_value + output.channels())]);
//                         value_io_outputs += 1;
//                     }
//                     SocketType::NodeRef => {}
//                 }
//             }

//             // finally, finally, finally!!!
//             let layout = nodes.remove(index).unwrap();

//             containers.push(NodeContainer {
//                 node: layout.node,
//                 to_input: layout.to_input,
//                 ins: Ins {
//                     midis: &midi_io[midi_io_input_index..(midi_io_input_index + midi_io_inputs)],
//                     values: &value_io[value_io_input_index..(value_io_input_index + value_io_inputs)],
//                     streams: &stream_io[stream_io_input_index..(stream_io_input_index + stream_io_inputs)],
//                 },
//                 outs: Outs {
//                     midis: &midi_io[midi_io_output_index..(midi_io_output_index + midi_io_outputs)],
//                     values: &value_io[value_io_output_index..(value_io_output_index + value_io_outputs)],
//                     streams: &stream_io[stream_io_output_index..(stream_io_output_index + stream_io_outputs)],
//                 },
//             });
//         }
//     });

//     Ok(())
// }

pub struct TraverserResult {
    pub errors_and_warnings: ErrorsAndWarnings,
    pub state_changes: Vec<(NodeIndex, NodeState)>,
    pub requested_state_updates: Vec<(NodeIndex, serde_json::Value)>,
    pub request_for_graph_state: bool,
}

#[derive(Debug, Clone)]
struct NodeAssociatedLocations {
    pub value_socket_to_index: Vec<(Socket, usize)>,
    pub stream_outputs_index: usize,
    pub stream_output_sockets: Vec<Socket>,
    pub midi_outputs_index: usize,
    pub midi_output_sockets: Vec<Socket>,
    pub value_outputs_index: usize,
    pub value_output_sockets: Vec<Socket>,
    pub vec_index: usize,
}

#[derive(Debug, Clone, Default)]
struct NodeTraversalWrapper {
    pub node: NodeVariant,
    /// A mapping of a value to input to its location
    pub values_to_input: SmallVec<[(usize, Primitive); 4]>,
}

pub struct NodeIo<'a> {
    stream_ins: &'a [&'a [&'a [Cell<f32>]]],
    stream_outs: &'a [&'a [&'a [Cell<f32>]]],
}

// impl BufferedTraverser {
//     pub fn traverse(
//         &mut self,
//         current_time: i64,
//         script_engine: &Engine,
//         resources: &Resources,
//         updated_node_states: Vec<(NodeIndex, serde_json::Value)>,
//         graph_state: Option<&BTreeMap<NodeIndex, NodeState>>,
//     ) -> TraverserResult {
//         let mut errors: Vec<(NodeIndex, NodeError)> = vec![];
//         let mut warnings: Vec<(NodeIndex, NodeWarning)> = vec![];

//         let mut state_changes: Vec<(NodeIndex, NodeState)> = vec![];

//         // used as a default pointer if a node doesn't have an input connected
//         let nothing_stream = vec![0.0_f32; self.buffer_size];
//         let nothing_midi = None;
//         let nothing_value = None;

//         let mut midi_mapping_i = 0;
//         let mut value_mapping_i = 0;
//         let mut stream_mapping_i = 0;

//         let mut resource_input_i = 0;

//         let mut midi_inputs: [&Option<MidiBundle>; BUFFER_SIZE] = [&nothing_midi; BUFFER_SIZE];
//         let mut value_inputs: [&Option<Primitive>; BUFFER_SIZE] = [&nothing_value; BUFFER_SIZE];
//         let mut value_staging: [Option<Primitive>; BUFFER_SIZE] = staging_values();
//         let mut stream_inputs: [&[f32]; BUFFER_SIZE] = [&nothing_stream; BUFFER_SIZE];
//         let mut resource_inputs: [MaybeUninit<Option<(ResourceIndex, &dyn Any)>>; BUFFER_SIZE] =
//             unsafe { MaybeUninit::uninit().assume_init() };

//         let mut midi_outputs_i = 0;
//         let mut value_outputs_i = 0;
//         let mut stream_outputs_i = 0;

//         let mut stream_outputs: [MaybeUninit<&mut [f32]>; BUFFER_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };

//         let mut requesting_graph_state = false;
//         let mut requested_state_updates = vec![];

//         // input updated node states
//         for (node_index, new_node_state) in updated_node_states.into_iter() {
//             let node = &mut self.nodes[self.node_to_location_mapping.get(&node_index).unwrap().vec_index];
//             node.node.set_state(new_node_state);
//         }

//         for (i, node) in self.nodes.iter_mut().enumerate() {
//             // input resources
//             for j in 0..self.resource_advance_by[i] {
//                 let (resource_id, possible_index) = &self.resource_tracking[resource_input_i];

//                 let possible_resource = possible_index.as_ref().and_then(|(resource_type, resource_index)| {
//                     resources
//                         .get_any(resource_type, *resource_index)
//                         .map(|resource| (resource_index, resource))
//                 });

//                 // does it exist at the index?
//                 let to_input = if let Some((index, resource)) = possible_resource {
//                     Some((*index, resource))
//                 } else {
//                     // else check to see if it has a new index
//                     if let Some(new_resource_index) = resources.get_resource_index(resource_id) {
//                         self.resource_tracking[resource_input_i].1 = Some(new_resource_index.clone());

//                         resources
//                             .get_any(&new_resource_index.0, new_resource_index.1)
//                             .map(|resource| (new_resource_index.1, resource))
//                     } else {
//                         // still doesn't exist
//                         None
//                     }
//                 };

//                 resource_inputs[j].write(to_input);

//                 resource_input_i += 1;
//             }

//             let midi_input_count = self.midi_advance_by[i].inputs;
//             let value_input_count = self.value_advance_by[i].inputs;
//             let stream_input_count = self.stream_advance_by[i].inputs;

//             let stream_output_count = self.stream_advance_by[i].outputs;
//             let value_output_count = self.value_advance_by[i].outputs;
//             let midi_output_count = self.midi_advance_by[i].outputs;

//             let midi_output_index = midi_outputs_i;
//             let value_output_index = value_outputs_i;

//             // clear last outputs (up to what we'll be using)
//             self.midi_outputs[midi_output_index..(midi_output_index + midi_output_count)].fill(None);
//             self.value_outputs[value_output_index..(value_output_index + value_output_count)].fill(None);

//             let midi_ptr = self.midi_outputs.as_mut_ptr();
//             let value_ptr = self.value_outputs.as_mut_ptr();
//             let value_staging_ptr = value_staging.as_mut_ptr();
//             let streams_ptr = self.stream_outputs.as_mut_ptr();

//             // set up midi and value inputs
//             for j in 0..midi_input_count {
//                 if let Some(input_index) = self.midi_input_mappings[midi_mapping_i] {
//                     // SAFETY: make sure we don't exceed the midi output's length
//                     assert!(input_index < self.midi_outputs.len());

//                     midi_inputs[j] = unsafe { &*midi_ptr.add(input_index) };
//                 } else {
//                     midi_inputs[j] = &nothing_midi;
//                 }

//                 midi_mapping_i += 1;
//             }

//             for j in 0..value_input_count {
//                 if let Some(input_index) = self.value_input_mappings[value_mapping_i] {
//                     value_inputs[j] = unsafe { &*value_ptr.add(input_index) };
//                 } else {
//                     value_inputs[j] = &nothing_value;
//                 }

//                 value_mapping_i += 1;
//             }

//             // override any values coming in with values from the user, if any
//             for (j, (input_at, override_input)) in node.values_to_input.drain(..).enumerate() {
//                 let staging_ref = unsafe { &mut *value_staging_ptr.add(j) };
//                 *staging_ref = Some(override_input);
//                 value_inputs[input_at] = staging_ref;
//             }

//             // build the list of input references from other nodes' outputs
//             for j in 0..stream_input_count {
//                 if let Some(input_index) = self.stream_input_mappings[stream_mapping_i] {
//                     // SAFETY: Make sure we don't have a slice exceed the length of the array
//                     assert!(input_index + self.buffer_size <= self.stream_outputs.len());

//                     stream_inputs[j] = unsafe { slice::from_raw_parts(streams_ptr.add(input_index), self.buffer_size) };
//                 } else {
//                     stream_inputs[j] = &nothing_stream;
//                 }

//                 stream_mapping_i += 1;
//             }

//             // ...and the list of output references
//             for j in 0..stream_output_count {
//                 let output_index = stream_outputs_i + j * self.buffer_size;

//                 // SAFETY: Make sure we don't have a slice exceed the length of the array
//                 assert!(output_index + self.buffer_size <= self.stream_outputs.len());

//                 stream_outputs[j]
//                     .write(unsafe { slice::from_raw_parts_mut(streams_ptr.add(output_index), self.buffer_size) });
//             }

//             // FINALLY
//             let res = node.node.process(
//                 NodeProcessContext {
//                     current_time,
//                     script_engine,
//                     resources,
//                     external_state: StateInterface {
//                         states: graph_state,
//                         request_node_states: &mut || requesting_graph_state = true,
//                         enqueue_state_updates: &mut |updates| requested_state_updates.extend(updates.into_iter()),
//                     },
//                 },
//                 // SAFETY: we've already initialized 0..inputs and 0..outputs above
//                 Ins {
//                     midis: &midi_inputs[0..midi_input_count],
//                     values: &value_inputs[0..value_input_count],
//                     streams: &stream_inputs[0..stream_input_count],
//                 },
//                 Outs {
//                     midis: unsafe {
//                         slice::from_raw_parts_mut(
//                             midi_ptr.add(midi_output_index),
//                             midi_output_index + midi_output_count,
//                         )
//                     },
//                     values: unsafe {
//                         slice::from_raw_parts_mut(
//                             value_ptr.add(value_output_index),
//                             value_output_index + value_output_count,
//                         )
//                     },
//                     streams: unsafe {
//                         mem::transmute::<_, &mut [&mut [f32]]>(&mut stream_outputs[0..stream_output_count])
//                     },
//                 },
//                 unsafe {
//                     mem::transmute::<
//                         &[MaybeUninit<Option<(ResourceIndex, &dyn Any)>>],
//                         &[Option<(ResourceIndex, &dyn Any)>],
//                     >(&resource_inputs[0..self.resource_advance_by[i]])
//                 },
//             );

//             match res {
//                 Ok(NodeOk {
//                     warnings: mut node_warnings,
//                     ..
//                 }) => {
//                     for warning in node_warnings.drain(..) {
//                         warnings.push((self.node_indexes[i], warning));
//                     }
//                 }
//                 Err(err) => {
//                     errors.push((self.node_indexes[i], err));
//                 }
//             }

//             midi_outputs_i += self.midi_advance_by[i].outputs;
//             value_outputs_i += self.value_advance_by[i].outputs;
//             stream_outputs_i += self.stream_advance_by[i].outputs * self.buffer_size;
//         }

//         for (vec_index, node_index) in &self.nodes_linked_to_ui {
//             if let Some(new_node_state) = self.nodes[*vec_index].node.get_state() {
//                 state_changes.push((*node_index, new_node_state));
//             }
//         }

//         TraverserResult {
//             errors_and_warnings: ErrorsAndWarnings { errors, warnings },
//             state_changes,
//             request_for_graph_state: requesting_graph_state,
//             requested_state_updates: requested_state_updates,
//         }
//     }

//     pub fn get_node_mut(&mut self, index_to_find: NodeIndex) -> Option<&mut NodeVariant> {
//         self.nodes
//             .iter_mut()
//             .zip(&self.node_indexes)
//             .find(|(_, index)| *index == &index_to_find)
//             .map(|(node, _)| &mut node.node)
//     }

//     pub fn input_value_default(
//         &mut self,
//         node_index: NodeIndex,
//         socket: &Socket,
//         value: Primitive,
//     ) -> Result<(), NodeError> {
//         let locations = self.node_to_location_mapping.get(&node_index);

//         if let Some(locations) = locations {
//             let value_index = locations
//                 .value_socket_to_index
//                 .iter()
//                 .find_map(|(possible_socket, index)| if possible_socket == socket { Some(*index) } else { None });

//             if let Some(value_index) = value_index {
//                 self.nodes[locations.vec_index]
//                     .values_to_input
//                     .push((value_index, value));

//                 Ok(())
//             } else {
//                 Err(NodeError::SocketDoesNotExist { socket: socket.clone() })
//             }
//         } else {
//             Err(NodeError::NodeDoesNotExist { node_index })
//         }
//     }
// }
