use semver::Version;
use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum EngineError {
    #[snafu(display("Device {device_name} doesn't exist"))]
    DeviceDoesNotExist { device_name: String },
    #[snafu(display("Device {device_name} already started"))]
    DeviceAlreadyStarted { device_name: String },
    #[snafu(display("Device {device_name} is missing in cpal device list"))]
    DeviceNotInCpalList { device_name: String },
    #[snafu(display("Error starting device: {source}"))]
    DeviceStartError { source: cpal::BuildStreamError },
    #[snafu(display("Audio parser error"))]
    AudioParserError,
    #[snafu(display("Node error: {source}"))]
    NodeError { source: node_engine::errors::NodeError },
    #[snafu(display("Cpal error: {source}"))]
    CpalError { source: Box<dyn std::error::Error> },
    #[snafu(display("Symphonia error: {source}"))]
    #[cfg(any(unix, windows))]
    SymphoniaError { source: symphonia::core::errors::Error },
    #[snafu(display("File error: {source}"))]
    FileError { source: std::io::Error },
    #[snafu(display("IO Error: {source}"))]
    IoError { source: std::io::Error },
    #[snafu(display("Json parser error: `{source}`"))]
    JsonParserError { source: serde_json::error::Error },
    #[snafu(display("Json parser error: `{source}` ({context})"))]
    JsonParserErrorInContext {
        source: serde_json::error::Error,
        context: String,
    },
    #[snafu(display("Property `{property_name}` missing or malformed"))]
    PropertyMissingOrMalformed { property_name: String },
    #[snafu(display("Version doesn't exist: {version}"))]
    VersionError { version: Version },
    #[snafu(display("Parser error: {source}"))]
    ParserError { source: serde_json::Error },
    #[snafu(display("TOML serialization error: {source}"))]
    TomlParserSerError { source: toml_edit::ser::Error },
    #[snafu(display("TOML deserialization error: {source}"))]
    TomlParserDeError { source: toml_edit::de::Error },
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
    },
}
