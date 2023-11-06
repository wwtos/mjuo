use std::{
    cell::Cell,
    iter::{repeat, repeat_with},
};

use common::alloc::{BuddyArena, SliceAlloc};
use ghost_cell::{GhostCell, GhostToken};
use rhai::Engine;
use sound_engine::{midi::messages::MidiMessage, SoundConfig};

use crate::{
    connection::Primitive,
    global_state::Resources,
    node::{Ins, MidiBorrow, NodeProcessContext, NodeRuntime, Outs, StateInterface},
    nodes::NodeVariant,
    traversal::calculate_traversal_order::NodeMappedIo,
};

use super::calculate_traversal_order::{Indexes, IoSpec};

pub fn start_traverser<'a, 'brand, 'arena>(
    config: SoundConfig,
    mut io_spec: IoSpec,
    indexes: &Indexes,
    stream_io: &'a [Cell<f32>],
    value_io: &'a [Cell<Primitive>],
    midi_io: &'a [GhostCell<'brand, Option<SliceAlloc<'arena, MidiMessage>>>],
    script_engine: &Engine,
    resources: &Resources,
    start_time: i64,
    token: &mut GhostToken<'brand>,
    arena: &'arena BuddyArena,
) {
    // first, set up refs
    let stream_buffers: Vec<&[Cell<f32>]> = stream_io.chunks_exact(config.buffer_size).collect();
    let mut stream_sockets: Vec<&[&[Cell<f32>]]> = vec![];

    let mut value_sockets: Vec<&[Cell<Primitive>]> = vec![];
    let mut midi_sockets: Vec<&[MidiBorrow]> = vec![];

    let stream_default_buffer: Vec<Cell<f32>> = repeat(Cell::new(0.0)).take(config.buffer_size).collect();
    let stream_default: Vec<&[Cell<f32>]> = repeat(&stream_default_buffer[..])
        .take(indexes.max_stream_channels)
        .collect();
    let midi_default: Vec<MidiBorrow> = repeat_with(|| GhostCell::new(None))
        .take(indexes.max_midi_channels)
        .collect();
    let value_default: Vec<Cell<Primitive>> = repeat(Cell::new(Primitive::None))
        .take(indexes.max_value_channels)
        .collect();

    for stream_config in &indexes.streams {
        if let Some(stream_config) = stream_config {
            stream_sockets.push(&stream_buffers[stream_config.clone()])
        } else {
            stream_sockets.push(&stream_default[..])
        }
    }

    for midi_config in &indexes.midis {
        if let Some(midi_config) = midi_config {
            midi_sockets.push(&midi_io[midi_config.clone()])
        } else {
            midi_sockets.push(&midi_default[..])
        }
    }

    for value_config in &indexes.values {
        if let Some(value_config) = value_config {
            value_sockets.push(&value_io[value_config.clone()])
        } else {
            value_sockets.push(&value_default[..])
        }
    }

    let mut node_and_io: Vec<(NodeVariant, NodeMappedIo)> = vec![];

    for index in io_spec.traversal_order.iter() {
        node_and_io.push((
            io_spec.nodes.remove(index).unwrap().node,
            indexes.node_io[index].clone(),
        ));
    }

    let mut current_time = start_time;

    for i in midi_sockets.iter() {
        for j in i.iter() {
            dbg!(j.borrow(token));
        }
    }

    for i in 0..10 {
        for (node, io) in node_and_io.iter_mut() {
            node.process(
                NodeProcessContext {
                    current_time,
                    resources,
                    script_engine,
                    external_state: StateInterface {
                        request_node_states: &mut || {},
                        enqueue_state_updates: &mut |_| {},
                        states: None,
                    },
                },
                Ins {
                    midis: &midi_sockets[io.midi_in.clone()],
                    values: &value_sockets[io.value_in.clone()],
                    streams: &stream_sockets[io.stream_in.clone()],
                },
                Outs {
                    midis: &midi_sockets[io.midi_out.clone()],
                    values: &value_sockets[io.value_out.clone()],
                    streams: &stream_sockets[io.stream_out.clone()],
                },
                token,
                arena,
                &[],
            )
            .unwrap();
        }

        current_time += config.buffer_size as i64;
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::Cell,
        collections::BTreeMap,
        iter::{repeat, repeat_with},
    };

    use common::alloc::BuddyArena;
    use ghost_cell::{GhostCell, GhostToken};
    use sound_engine::SoundConfig;

    use crate::{
        connection::{Primitive, Socket, SocketType},
        global_state::Resources,
        graph_manager::GraphManager,
        node::MidiBorrow,
        traversal::calculate_traversal_order::{calc_indexes, get_node_io_needed},
    };

    use super::start_traverser;

    #[test]
    fn test_layout() {
        let mut manager = GraphManager::new();
        let (graph_index, _) = manager.new_graph().unwrap();
        let graph = manager.get_graph_mut(graph_index).unwrap();

        let (gain, _) = graph.add_node("GainNode").unwrap().value;
        let (midi, _) = graph.add_node("MidiToValuesNode").unwrap().value;
        let (osc, _) = graph.add_node("OscillatorNode").unwrap().value;

        graph
            .connect(
                midi,
                &Socket::Simple("frequency".into(), SocketType::Value, 1),
                osc,
                &Socket::Simple("frequency".into(), SocketType::Value, 1),
            )
            .unwrap();

        graph
            .connect(
                osc,
                &Socket::Simple("audio".into(), SocketType::Stream, 1),
                gain,
                &Socket::Simple("audio".into(), SocketType::Stream, 1),
            )
            .unwrap();

        let needed_io = get_node_io_needed(
            manager.get_graph(graph_index).unwrap(),
            BTreeMap::new(),
            &SoundConfig {
                sample_rate: 48_000,
                buffer_size: 512,
            },
            &mut rhai::Engine::new(),
            &Resources::default(),
            0,
            &manager,
            1,
        )
        .unwrap();
        let indexes = calc_indexes(&needed_io, graph_index, &manager).unwrap();

        let arena = BuddyArena::new(1_000_000);

        GhostToken::new(|mut token| {
            let stream_io: Vec<Cell<f32>> = repeat(Cell::new(0.0)).take(8 * indexes.stream_count).collect();
            let midi_io: Vec<MidiBorrow> = repeat_with(|| GhostCell::new(None)).take(indexes.midi_count).collect();
            let value_io: Vec<Cell<Primitive>> = repeat(Cell::new(Primitive::None)).take(indexes.value_count).collect();

            start_traverser(
                SoundConfig {
                    sample_rate: 48_000,
                    buffer_size: 8,
                },
                needed_io,
                &indexes,
                &stream_io[..],
                &value_io[..],
                &midi_io[..],
                &rhai::Engine::new(),
                &Resources::default(),
                0,
                &mut token,
                &arena,
            );
        });
    }
}
