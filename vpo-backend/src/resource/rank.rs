use std::{fs::File, io::Read, path::PathBuf};

use resource_manager::{IOSnafu, LoadingError, TomlParserDeSnafu};
use snafu::ResultExt;
use sound_engine::sampling::rank::Rank;

pub fn load_rank<T>(byte_stream: &mut T) -> Result<Rank, LoadingError>
where
    T: Read,
{
    let mut data = String::new();
    byte_stream.read_to_string(&mut data).context(IOSnafu)?;

    toml::from_str(&data).context(TomlParserDeSnafu)
}

pub fn load_rank_from_file(path: PathBuf) -> Result<Rank, LoadingError> {
    let mut file = File::open(path).context(IOSnafu)?;

    load_rank(&mut file)
}
