use common::SeaHashMap;

use crate::property::Property;

pub trait UiNode {
    fn has_new_state(&self) -> bool;

    fn get_new_state(&self) -> SeaHashMap<String, Property>;

    fn apply_state(&mut self, state: SeaHashMap<String, Property>);
}
