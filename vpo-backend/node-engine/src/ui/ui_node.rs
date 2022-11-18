use std::collections::HashMap;

use crate::property::Property;

pub trait UiNode {
    fn has_new_state(&self) -> bool;

    fn get_new_state(&self) -> HashMap<String, Property>;

    fn apply_state(&mut self, state: HashMap<String, Property>);
}
