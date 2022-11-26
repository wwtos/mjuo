use std::{fs, path::PathBuf};

use node_engine::errors::{IOSnafu, JsonParserSnafu, NodeError};
use serde_json::Value;
use snafu::ResultExt;

pub fn migrate(project: PathBuf) -> Result<(), NodeError> {
    let json_raw = fs::read_to_string(project.join("state.json")).context(IOSnafu)?;
    let mut json: Value = serde_json::from_str(&json_raw).context(JsonParserSnafu)?;

    let graphs =
        json["state"]["graph_manager"]["node_graphs"]
            .as_object_mut()
            .ok_or(NodeError::PropertyMissingOrMalformed {
                property_name: "state.graph_manager.node_graphs".into(),
            })?;

    graphs
        .iter_mut()
        .map(|(k, v)| {
            let nodes = v["graph"]["nodes"]
                .as_array_mut()
                .ok_or(NodeError::PropertyMissingOrMalformed {
                    property_name: format!("state.graph_manager.node_graphs.{}.graph.nodes", k),
                })?;

            nodes
                .iter_mut()
                .enumerate()
                .filter(|(_, node)| node["variant"].as_str().unwrap() == "Some")
                .map(|(node_i, node)| {
                    let node_rows =
                        node["data"][0]["node_rows"]
                            .as_array_mut()
                            .ok_or(NodeError::PropertyMissingOrMalformed {
                                property_name: format!(
                                    "state.graph_manager.node_graphs.{}.graph.nodes[{}].data[0].node_rows",
                                    k, node_i
                                ),
                            })?;

                    node_rows
                        .iter_mut()
                        .enumerate()
                        .map(|(row_i, row)| {
                            let row_variant = row["variant"].as_str().ok_or(NodeError::PropertyMissingOrMalformed { property_name: format!(
                                    "state.graph_manager.node_graphs.{}.graph.nodes[{}].data.node_rows[{}]",
                                    k, node_i, row_i
                                ),
                            })?;

                            if row_variant == "NodeRowInput" || row_variant == "NodeRowOutput" {
                                if !matches!(row["data"], Value::Array(_)) {
                                    // if it's not an array, the migration needs to be applied
                                    row["data"] = Value::Array(vec![row["data"].take(), Value::Bool(false)]);
                                }
                            } else if let "StreamInput" | "MidiInput" | "ValueInput" |
                                          "StreamOutput" | "MidiOutput" | "ValueOutput" = row_variant {
                                if let Value::Array(ref mut row_data) = row["data"] {
                                    if row_data.len() == 2 {
                                        // migration needs to be applied
                                        row_data.push(Value::Bool(false));
                                    }
                                } else {
                                    return Err(NodeError::PropertyMissingOrMalformed {
                                        property_name: format!(
                                            "state.graph_manager.node_graphs.{}.graph.nodes[{}].data.node_rows[{}]",
                                            k, node_i, row_i
                                        ),
                                    });
                                }
                            }

                            Ok(())
                        })
                        .collect::<Result<(), NodeError>>()?;

                    let node_rows =
                        node["data"][0]["node_rows"]
                            .as_array_mut()
                            .ok_or(NodeError::PropertyMissingOrMalformed {
                                property_name: format!(
                                    "state.graph_manager.node_graphs.{}.graph.nodes[{}].data[0].node_rows",
                                    k, node_i
                                ),
                            })?;

                    node_rows
                        .iter_mut()
                        .enumerate()
                        .map(|(row_i, row)| {
                            let row_variant = row["variant"].as_str().ok_or(NodeError::PropertyMissingOrMalformed {
                                property_name: format!(
                                    "state.graph_manager.node_graphs.{}.graph.nodes[{}].data.node_rows[{}]",
                                    k, node_i, row_i
                                ),
                            })?;

                            if row_variant == "NodeRowInput" || row_variant == "NodeRowOutput" {
                                if !matches!(row["data"], Value::Array(_)) {
                                    // if it's not an array, the migration needs to be applied
                                    row["data"] = Value::Array(vec![row["data"].take(), Value::Bool(false)]);
                                }
                            } else {
                                if let Value::Array(ref mut row_data) = row["data"] {
                                    if row_data.len() == 2 {
                                        // migration needs to be applied
                                        row_data.push(Value::Bool(false));
                                    }
                                } else {
                                    return Err(NodeError::PropertyMissingOrMalformed {
                                        property_name: format!(
                                            "state.graph_manager.node_graphs.{}.graph.nodes[{}].data.node_rows[{}]",
                                            k, node_i, row_i
                                        ),
                                    });
                                }
                            }

                            Ok(())
                        })
                        .collect::<Result<(), NodeError>>()?;

                        let default_overrides =
                        node["data"][0]["default_overrides"]
                            .as_array_mut()
                            .ok_or(NodeError::PropertyMissingOrMalformed {
                                property_name: format!(
                                    "state.graph_manager.node_graphs.{}.graph.nodes[{}].data[0].default_overrides",
                                    k, node_i
                                ),
                            })?;

                    default_overrides
                        .iter_mut()
                        .enumerate()
                        .map(|(row_i, row)| {
                            let row_variant = row["variant"].as_str().ok_or(NodeError::PropertyMissingOrMalformed {
                                property_name: format!(
                                    "state.graph_manager.node_graphs.{}.graph.nodes[{}].data.default_overrides[{}]",
                                    k, node_i, row_i
                                ),
                            })?;

                            if row_variant == "NodeRowInput" || row_variant == "NodeRowOutput" {
                                if !matches!(row["data"], Value::Array(_)) {
                                    // if it's not an array, the migration needs to be applied
                                    row["data"] = Value::Array(vec![row["data"].take(), Value::Bool(false)]);
                                }
                            } else {
                                if let Value::Array(ref mut row_data) = row["data"] {
                                    if row_data.len() == 2 {
                                        // migration needs to be applied
                                        row_data.push(Value::Bool(false));
                                    }
                                } else {
                                    return Err(NodeError::PropertyMissingOrMalformed {
                                        property_name: format!(
                                            "state.graph_manager.node_graphs.{}.graph.nodes[{}].data.default_overrides[{}]",
                                            k, node_i, row_i
                                        ),
                                    });
                                }
                            }

                            Ok(())
                        })
                        .collect::<Result<(), NodeError>>()?;

                    Ok(())
                })
                .collect::<Result<(), NodeError>>()?;

            Ok(())
        })
        .collect::<Result<(), NodeError>>()?;

    json["version"] = Value::String("0.4.0".into());

    fs::write(
        project.join("state.json"),
        serde_json::to_string_pretty(&json).context(JsonParserSnafu)?,
    )
    .context(IOSnafu)?;

    Ok(())
}
