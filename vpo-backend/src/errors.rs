use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum EngineError {
    #[snafu(display("Audio parser error (symphonia): {source}"))]
    SymphoniaError { source: symphonia::core::errors::Error },
    #[snafu(display("Audio parser error"))]
    AudioParserError,
    #[snafu(display("Loading error: "))]
    LoadingError { source: resource_manager::LoadingError },
    #[snafu(display("Node error: "))]
    NodeError { source: node_engine::errors::NodeError },
}
