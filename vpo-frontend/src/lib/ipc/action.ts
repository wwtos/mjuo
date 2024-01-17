import type { Graph } from "$lib/ddgg/graph";
import type { GlobalState, Resources } from "$lib/node-engine/global_state";
import type { NodeInstance } from "$lib/node-engine/node";
import type { NodeConnection } from "$lib/node-engine/node_graph";
import type { DiscriminatedUnion } from "$lib/util/discriminated-union";

export type IpcAction = DiscriminatedUnion<
    "action",
    {
        "graph/updateGraph": {
            payload: {
                graphIndex: string;
                nodes: Graph<NodeInstance, NodeConnection>;
            };
        };
        "state/updateState": {
            payload: GlobalState;
        };
        "state/updateResources": {
            payload: string
        };
        "toast/error": {
            payload: string;
        };
        "clipboard/set": {
            payload: string;
        };
    }
>;
