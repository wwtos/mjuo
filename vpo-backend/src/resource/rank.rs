use std::{fs::read_to_string, path::PathBuf};

use resource_manager::{IOSnafu, LoadingError, TomlParserDeSnafu};
use snafu::ResultExt;
use sound_engine::sampling::rank::Rank;

pub fn parse_rank(config: &str) -> Result<Rank, LoadingError> {
    toml::from_str(&config).context(TomlParserDeSnafu)
}

pub fn load_rank_from_file(path: PathBuf) -> Result<Rank, LoadingError> {
    let mut file = read_to_string(path).context(IOSnafu)?;

    parse_rank(&mut file)
}
