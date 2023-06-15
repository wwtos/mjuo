import type { VertexIndex } from "$lib/ddgg/graph";
import type { DiscriminatedUnion } from "$lib/util/discriminated-union";
import type { Socket } from "./connection";
import type { GlobalNodeIndex } from "./graph_manager";
import type { NodeRow } from "./node";
import type { Property } from "./property";

export type Action = DiscriminatedUnion<"variant", {
    AddNode: {
        graph: VertexIndex,
        nodeType: string,
    },
    ConnectNodes: {
        from: GlobalNodeIndex,
        to: GlobalNodeIndex,
        data: {
            fromSocket: Socket,
            toSocket: Socket,
        }
    },
    DisconnectNodes: {
        from: GlobalNodeIndex,
        to: GlobalNodeIndex,
        data: {
            fromSocket: Socket,
            toSocket: Socket,
        }
    },
    RemoveNode: {
        index: GlobalNodeIndex
    },
    ChangeNodeProperties: {
        index: GlobalNodeIndex,
        props: {
            [key: string]: Property
        }
    },
    ChangeNodeUiData: {
        index: GlobalNodeIndex,
        data: {
            [key: string]: any
        },
    },
    ChangeNodeOverrides: {
        index: GlobalNodeIndex,
        overrides: Array<NodeRow>
    }
}>;