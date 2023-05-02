import type { Index } from "$lib/ddgg/gen_vec";
import type { Graph } from "$lib/ddgg/graph";
import type { GlobalState } from "$lib/node-engine/global_state";
import type { NodeWrapper } from "$lib/node-engine/node";
import type { NodeConnection } from "$lib/node-engine/node_graph";
import type { DiscriminatedUnion } from "$lib/util/discriminated-union";

export type IpcAction = DiscriminatedUnion<"action", {
    "graph/updateGraph": {
        payload: {
            graphIndex: Index,
            nodes: Graph<NodeWrapper, NodeConnection>
        }
    },
    "state/updateGlobalState": {
        payload: GlobalState
    },
    "registry/updateRegistry": {
        payload: object
    },
    "toast/error": {
        payload: string
    }
}>;