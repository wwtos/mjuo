use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::Path,
};

pub trait Asset {
    type AssetType;

    fn convert_asset<R: Read>(stream: R) -> Self::AssetType;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AssetIndex {
    pub index: usize,
    pub generation: u32,
}

pub enum PossibleAsset<A: Asset> {
    Some(A::AssetType, u32),
    None(u32),
}

pub struct AssetManager<A: Asset> {
    assets: Vec<PossibleAsset<A>>,
    asset_mapping: HashMap<String, AssetIndex>,
}

impl<A: Asset> AssetManager<A> {
    pub fn new() -> AssetManager<A> {
        AssetManager {
            assets: Vec::new(),
            asset_mapping: HashMap::new(),
        }
    }

    fn add_asset(&mut self, asset: A::AssetType) -> AssetIndex {
        // check if there's an opening
        let possible_opening = self.assets.iter().position(|asset| {
            if let PossibleAsset::Some(..) = asset {
                true
            } else {
                false
            }
        });

        // put the new asset in the opening
        if let Some(opening) = possible_opening {
            let new_generation = match self.assets[opening] {
                PossibleAsset::Some(..) => unreachable!(),
                PossibleAsset::None(generation) => generation + 1,
            };

            self.assets[opening] = PossibleAsset::Some(asset, new_generation);

            AssetIndex {
                index: opening,
                generation: new_generation,
            }
        } else {
            // else, expand the asset length
            let index = self.assets.len();
            let new_generation = 0;

            self.assets.push(PossibleAsset::Some(asset, new_generation));

            AssetIndex {
                index,
                generation: new_generation,
            }
        }
    }

    pub fn request_asset(&mut self, key: String, location: &Path) -> Result<AssetIndex, io::Error> {
        // check if we've loaded this asset already
        if let Some(asset_index) = self.asset_mapping.get(&key) {
            Ok(*asset_index)
        } else {
            // else, load and register it
            let stream = File::open(location)?;

            let new_asset = A::convert_asset(stream);
            let asset_index = self.add_asset(new_asset);

            // now add the mapping
            self.asset_mapping.insert(key, asset_index);

            Ok(asset_index)
        }
    }
}
