use std::{collections::HashMap, hash::BuildHasherDefault};

use seahash::SeaHasher;

use crate::property::Property;

pub trait UiNode {
    fn has_new_state(&self) -> bool;

    fn get_new_state(&self) -> HashMap<String, Property, BuildHasherDefault<SeaHasher>>;

    fn apply_state(&mut self, state: HashMap<String, Property, BuildHasherDefault<SeaHasher>>);
}
