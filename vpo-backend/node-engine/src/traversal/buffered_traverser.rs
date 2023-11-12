use std::{
    cell::UnsafeCell,
    iter::{repeat, repeat_with},
};

use common::alloc::BuddyArena;
use rhai::Engine;
use self_cell::self_cell;
use sound_engine::SoundConfig;

use crate::{
    connection::Primitive,
    global_state::Resources,
    midi_store::MidiStore,
    node::{Ins, MidiStoreInterface, MidisIndex, NodeProcessContext, NodeRuntime, Outs, StateInterface},
    nodes::NodeVariant,
    traversal::calculate_traversal_order::NodeMappedIo,
};

use super::calculate_traversal_order::{Indexes, IoSpec};

struct BufferChunks<'a>(Vec<&'a [UnsafeCell<f32>]>);
self_cell!(
    struct ChunkedBuffer {
        owner: Vec<UnsafeCell<f32>>,

        #[covariant]
        dependent: BufferChunks,
    }
);

impl ChunkedBuffer {
    fn chunks(&self) -> &[&[UnsafeCell<f32>]] {
        &self.borrow_dependent().0[..]
    }
}

fn build_chunked_buffer(buffer: Vec<UnsafeCell<f32>>, chunk_size: usize) -> ChunkedBuffer {
    ChunkedBuffer::new(buffer, |buffer| BufferChunks(buffer.chunks_exact(chunk_size).collect()))
}

struct TraverserIo {
    stream_io: ChunkedBuffer,
    value_io: Vec<UnsafeCell<Primitive>>,
    midi_io: Vec<UnsafeCell<Option<MidisIndex>>>,
    stream_default: ChunkedBuffer,
    midi_default: Vec<UnsafeCell<Option<MidisIndex>>>,
    value_default: Vec<UnsafeCell<Primitive>>,
}

struct TraverserRefs<'io> {
    stream_sockets: Vec<&'io [&'io [UnsafeCell<f32>]]>,
    value_sockets: Vec<&'io [UnsafeCell<Primitive>]>,
    midi_sockets: Vec<&'io [UnsafeCell<Option<MidisIndex>>]>,
}

self_cell!(
    struct TraverserIoAndRefs {
        owner: TraverserIo,

        #[covariant]
        dependent: TraverserRefs,
    }
);

fn build_io(config: SoundConfig, indexes: &Indexes, arena: BuddyArena) -> TraverserIo {
    let stream_io = build_chunked_buffer(
        repeat_with(|| UnsafeCell::new(0.0))
            .take(config.buffer_size * indexes.stream_count)
            .collect(),
        config.buffer_size,
    );
    let value_io = repeat_with(|| UnsafeCell::new(Primitive::None))
        .take(indexes.value_count)
        .collect();
    let midi_io = repeat_with(|| UnsafeCell::new(None)).take(indexes.midi_count).collect();

    let stream_default = ChunkedBuffer::new(
        repeat_with(|| UnsafeCell::new(0.0)).take(config.buffer_size).collect(),
        |buffer| BufferChunks(repeat(&buffer[..]).take(indexes.max_stream_channels).collect()),
    );
    let midi_default = repeat_with(|| UnsafeCell::new(None))
        .take(indexes.max_midi_channels)
        .collect();
    let value_default = repeat_with(|| UnsafeCell::new(Primitive::None))
        .take(indexes.max_value_channels)
        .collect();

    TraverserIo {
        stream_io,
        value_io,
        midi_io,
        stream_default,
        midi_default,
        value_default,
    }
}

fn build_refs<'io>(io: &'io TraverserIo, config: SoundConfig, indexes: &Indexes) -> TraverserRefs<'io> {
    let mut stream_sockets: Vec<&[&[UnsafeCell<f32>]]> = vec![];
    let mut midi_sockets: Vec<&[UnsafeCell<Option<MidisIndex>>]> = vec![];
    let mut value_sockets: Vec<&[UnsafeCell<Primitive>]> = vec![];

    for stream_config in &indexes.streams {
        if let Some(stream_config) = stream_config {
            stream_sockets.push(&io.stream_io.chunks()[stream_config.clone()])
        } else {
            stream_sockets.push(&io.stream_default.chunks()[..])
        }
    }

    for midi_config in &indexes.midis {
        if let Some(midi_config) = midi_config {
            midi_sockets.push(&io.midi_io[midi_config.clone()])
        } else {
            midi_sockets.push(&io.midi_default[..])
        }
    }

    for value_config in &indexes.values {
        if let Some(value_config) = value_config {
            value_sockets.push(&io.value_io[value_config.clone()])
        } else {
            value_sockets.push(&io.value_default[..])
        }
    }

    TraverserRefs {
        stream_sockets,
        midi_sockets,
        value_sockets,
    }
}

self_cell!(
    struct IoAndRefs {
        owner: TraverserIo,

        #[covariant]
        dependent: TraverserRefs,
    }
);

pub struct BufferedTraverser {
    io_and_refs: IoAndRefs,
    store: MidiStore,
    config: SoundConfig,
    engine: Engine,
    time: i64,
}

pub fn build_traverser<'a, 'arena>(
    config: SoundConfig,
    mut io_spec: IoSpec,
    indexes: &Indexes,
    script_engine: &Engine,
    resources: &Resources,
    start_time: i64,
    arena: BuddyArena,
) {
    let io = build_io(config.clone(), indexes, arena);
    let io_and_refs = IoAndRefs::new(io, |io| build_refs(io, config.clone(), indexes));

    let mut node_and_io: Vec<(NodeVariant, NodeMappedIo)> = vec![];

    for index in io_spec.traversal_order.iter() {
        node_and_io.push((
            io_spec.nodes.remove(index).unwrap().node,
            indexes.node_io[index].clone(),
        ));
    }

    let TraverserRefs {
        stream_sockets,
        value_sockets,
        midi_sockets,
    } = &io_and_refs.borrow_dependent();

    let mut current_time = start_time;

    let mut store = MidiStore::new(1_000_000, 4096);

    for i in midi_sockets {
        for j in i.iter() {
            dbg!(j);
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
                unsafe {
                    Ins::new(
                        &midi_sockets[io.midi_in.clone()],
                        &value_sockets[io.value_in.clone()],
                        &stream_sockets[io.stream_in.clone()],
                    )
                },
                unsafe {
                    Outs::new(
                        &midi_sockets[io.midi_out.clone()],
                        &value_sockets[io.value_out.clone()],
                        &stream_sockets[io.stream_out.clone()],
                    )
                },
                &mut MidiStoreInterface::new(&mut store),
                &[],
            )
            .unwrap();
        }

        current_time += config.buffer_size as i64;
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::UnsafeCell, collections::BTreeMap, iter::repeat_with};

    use common::alloc::BuddyArena;
    use sound_engine::SoundConfig;

    use crate::{
        connection::{Primitive, Socket, SocketType},
        global_state::Resources,
        graph_manager::GraphManager,
        node::MidisIndex,
        traversal::calculate_traversal_order::{calc_indexes, get_node_io_needed},
    };

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

        let stream_io: Vec<UnsafeCell<f32>> = repeat_with(|| UnsafeCell::new(0.0))
            .take(8 * indexes.stream_count)
            .collect();
        let midi_io: Vec<UnsafeCell<Option<MidisIndex>>> =
            repeat_with(|| UnsafeCell::new(None)).take(indexes.midi_count).collect();
        let value_io: Vec<UnsafeCell<Primitive>> = repeat_with(|| UnsafeCell::new(Primitive::None))
            .take(indexes.value_count)
            .collect();

        // start_traverser(
        //     SoundConfig {
        //         sample_rate: 48_000,
        //         buffer_size: 8,
        //     },
        //     needed_io,
        //     &indexes,
        //     &stream_io[..],
        //     &value_io[..],
        //     &midi_io[..],
        //     &rhai::Engine::new(),
        //     &Resources::default(),
        //     0,
        //     &arena,
        // );
    }
}
