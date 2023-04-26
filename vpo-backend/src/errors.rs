use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum EngineError {
    #[snafu(display("Audio parser error"))]
    AudioParserError,
    #[snafu(display("Loading error: "))]
    LoadingError { source: resource_manager::LoadingError },
    #[snafu(display("Node error: "))]
    NodeError { source: node_engine::errors::NodeError },
    #[snafu(display("Cpal error: {source}"))]
    CpalError { source: Box<dyn std::error::Error> },
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
    },
}
