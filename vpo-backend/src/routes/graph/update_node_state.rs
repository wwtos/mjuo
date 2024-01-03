use node_engine::node::NodeIndex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;

use crate::{
    errors::{EngineError, JsonParserSnafu},
    routes::prelude::*,
    routes::RouteReturn,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Payload {
    updated_states: Vec<(NodeIndex, Value)>,
}

pub fn route(mut state: RouteState) -> Result<RouteReturn, EngineError> {
    let payload: Payload = serde_json::from_value(state.msg["payload"].take()).context(JsonParserSnafu)?;

    Ok(RouteReturn {
        engine_updates: vec![ToAudioThread::NewNodeStates(payload.updated_states)],
        new_project: false,
    })
}
