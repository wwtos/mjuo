use std::{fs::File, io::Read, path::PathBuf};

use resource_manager::{IOSnafu, LoadingError, TomlParserDeSnafu};
use snafu::ResultExt;
use sound_engine::sampling::rank::Rank;

pub fn load_rank(path: PathBuf) -> Result<Rank, LoadingError> {
    let mut file = File::open(path).context(IOSnafu)?;
    let mut data = String::new();
    file.read_to_string(&mut data).context(IOSnafu)?;

    toml::from_str(&data).context(TomlParserDeSnafu)
}
