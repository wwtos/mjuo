use std::{collections::HashMap, hash::BuildHasherDefault};

use seahash::SeaHasher;

pub mod alloc;
pub mod osc;
pub mod resource_manager;
pub mod traits;

pub type SeaHashMap<K, V> = HashMap<K, V, BuildHasherDefault<SeaHasher>>;
