use std::{collections::HashMap, io, path::Path};

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum LoadingError {
    #[snafu(display("IO error: {source}"))]
    IOError { source: io::Error },
    #[snafu(display("Unknown error: {source}"))]
    Other { source: Box<dyn std::error::Error> },
}

pub trait Resource {
    fn load_resource(path: &Path) -> Result<Self, LoadingError>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResourceIndex {
    pub index: usize,
    pub generation: u32,
}

pub enum PossibleResource<A: Resource> {
    Some(A, u32),
    None(u32),
}

#[derive(Default)]
pub struct ResourceManager<A: Resource> {
    assets: Vec<PossibleResource<A>>,
    asset_mapping: HashMap<String, ResourceIndex>,
}

impl<A: Resource> ResourceManager<A> {
    pub fn new() -> ResourceManager<A> {
        ResourceManager {
            assets: Vec::new(),
            asset_mapping: HashMap::new(),
        }
    }

    fn add_asset(&mut self, asset: A) -> ResourceIndex {
        // check if there's an opening
        let possible_opening = self.assets.iter().position(|asset| {
            if let PossibleResource::Some(..) = asset {
                false
            } else {
                true
            }
        });

        // put the new asset in the opening
        if let Some(opening) = possible_opening {
            let new_generation = match self.assets[opening] {
                PossibleResource::Some(..) => unreachable!(),
                PossibleResource::None(generation) => generation + 1,
            };

            self.assets[opening] = PossibleResource::Some(asset, new_generation);

            ResourceIndex {
                index: opening,
                generation: new_generation,
            }
        } else {
            // else, expand the asset length
            let index = self.assets.len();
            let new_generation = 0;

            self.assets.push(PossibleResource::Some(asset, new_generation));

            ResourceIndex {
                index,
                generation: new_generation,
            }
        }
    }

    pub fn get_index(&self, key: &str) -> Option<ResourceIndex> {
        self.asset_mapping.get(key).map(|x| *x)
    }

    pub fn request_asset(&mut self, key: String, location: &Path) -> Result<ResourceIndex, LoadingError> {
        // check if we've loaded this asset already
        if let Some(asset_index) = self.asset_mapping.get(&key) {
            Ok(*asset_index)
        } else {
            // else, load and register it
            let new_asset = A::load_resource(location)?;
            let asset_index = self.add_asset(new_asset);

            // now add the mapping
            self.asset_mapping.insert(key, asset_index);

            Ok(asset_index)
        }
    }

    pub fn borrow_asset(&self, index: ResourceIndex) -> Option<&A> {
        if index.index >= self.assets.len() {
            None
        } else {
            match &self.assets[index.index] {
                PossibleResource::Some(asset, generation) => {
                    if index.generation == *generation {
                        Some(&asset)
                    } else {
                        None
                    }
                }
                PossibleResource::None(_) => None,
            }
        }
    }

    pub fn clear(&mut self) {
        self.asset_mapping.clear();
        self.assets.clear();
    }
}
