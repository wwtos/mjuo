use std::{collections::HashMap, hash::BuildHasherDefault};

use seahash::SeaHasher;

pub mod alloc;
pub mod osc;
pub mod osc_midi;
pub mod resource_manager;
pub mod traits;

pub type SeaHashMap<K, V> = HashMap<K, V, BuildHasherDefault<SeaHasher>>;
