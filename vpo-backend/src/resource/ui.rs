use crate::errors::{EngineError, IoSnafu};
use std::path::Path;

pub fn load_ui_from_file(path: &Path) -> Result<String, EngineError> {
    use std::fs::read_to_string;

    use snafu::ResultExt;

    let file = read_to_string(path).context(IoSnafu)?;

    Ok(file)
}
