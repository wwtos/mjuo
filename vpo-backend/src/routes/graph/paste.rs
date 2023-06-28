use std::collections::{BTreeMap, HashSet};

use node_engine::{
    graph_manager::{GlobalNodeIndex, GraphIndex},
    node_graph::NodeConnectionData,
    node_wrapper::NodeWrapper,
    state::{Action, ActionInvalidation},
};
use petgraph::Graph;
use serde::Deserialize;
use snafu::ResultExt;

use crate::{
    errors::{JsonParserSnafu, NodeSnafu},
    routes::prelude::*,
    util::send_graph_updates,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    graph_index: GraphIndex,
    clipboard: String,
}

pub fn route(mut state: RouteState) -> Result<RouteReturn, EngineError> {
    let payload: Payload = serde_json::from_value(state.msg["payload"].take()).context(JsonParserSnafu)?;
    let mini_graph: Graph<NodeWrapper, NodeConnectionData> =
        serde_json::from_str(&payload.clipboard).context(JsonParserSnafu)?;

    let mut mapping = BTreeMap::new();

    let mut invalidations = vec![];

    // create all the nodes
    let mut first_time = true;
    for index in mini_graph.node_indices() {
        let node = &mini_graph[index];

        // create the node and get its index
        let new_invalidations = state
            .state
            .commit(
                ActionBundle {
                    actions: vec![Action::CreateNode {
                        graph: payload.graph_index,
                        node_type: node.get_node_type(),
                        ui_data: node.get_ui_data().clone(),
                    }],
                },
                !first_time,
            )
            .context(NodeSnafu)?;

        let new_node_index = new_invalidations
            .iter()
            .find_map(|invalidation| match invalidation {
                ActionInvalidation::NewNode(index) => Some(*index),
                _ => None,
            })
            .expect("node created invalidation");

        mapping.insert(index, new_node_index);

        invalidations.extend(new_invalidations.into_iter());
        invalidations.extend(
            state
                .state
                .commit(
                    ActionBundle {
                        actions: vec![
                            Action::ChangeNodeProperties {
                                index: new_node_index,
                                props: node.get_properties().clone(),
                            },
                            Action::ChangeNodeOverrides {
                                index: new_node_index,
                                overrides: node.get_default_overrides().clone(),
                            },
                        ],
                    },
                    true,
                )
                .context(NodeSnafu)?
                .into_iter(),
        );

        first_time = false;
    }

    // now connect all the nodes
    for edge_index in mini_graph.edge_indices() {
        let (from, to) = &mini_graph.edge_endpoints(edge_index).expect("edge to exist");
        let data = &mini_graph[edge_index];

        invalidations.extend(
            state
                .state
                .commit(
                    ActionBundle {
                        actions: vec![Action::ConnectNodes {
                            graph: payload.graph_index,
                            from: mapping[from].node_index,
                            to: mapping[to].node_index,
                            data: data.clone(),
                        }],
                    },
                    true,
                )
                .context(NodeSnafu)?
                .into_iter(),
        );
    }

    let mut touched_graphs = HashSet::new();
    for invalidation in &invalidations {
        match invalidation {
            ActionInvalidation::GraphReindexNeeded(index)
            | ActionInvalidation::GraphModified(index)
            | ActionInvalidation::NewDefaults(GlobalNodeIndex { graph_index: index, .. }, _)
            | ActionInvalidation::NewNode(GlobalNodeIndex { graph_index: index, .. }) => {
                touched_graphs.insert(index);
            }
            ActionInvalidation::None => {}
        }
    }

    for graph_index in touched_graphs {
        send_graph_updates(state.state, *graph_index, state.to_server)?;
    }

    Ok(RouteReturn {
        engine_updates: state
            .state
            .invalidations_to_engine_updates(invalidations, state.global_state)
            .context(NodeSnafu)?,
        new_project: false,
    })
}
