use resource_manager::ResourceIndex;

use super::sample_player::SamplePlayer;

struct Voice {
    player: SamplePlayer,
}

pub struct RankPlayer {
    polyphony: usize,
    voices: Vec<Voice>,
    note_map: Vec<Option<ResourceIndex>>,
}
