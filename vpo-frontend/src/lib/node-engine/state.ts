import type { VertexIndex } from "$lib/ddgg/graph";
import type { DiscriminatedUnion } from "$lib/util/discriminated-union";
import type { Socket } from "./connection";
import type { GlobalNodeIndex } from "./graph_manager";
import type { IoRoutes } from "./io_routing";
import type { NodeRow } from "./node";
import type { Property } from "./property";


export type Action = DiscriminatedUnion<"variant", {
    CreateNode: {
        data: {
            graph: VertexIndex,
            nodeType: string,
            uiData: {
                [key: string]: any
            },
        }
    },
    ConnectNodes: {
        data: {
            graph: VertexIndex,
            from: VertexIndex,
            to: VertexIndex,
            data: {
                fromSocket: Socket,
                toSocket: Socket,
            }
        }
    },
    DisconnectNodes: {
        data: {
            graph: VertexIndex,
            from: VertexIndex,
            to: VertexIndex,
            data: {
                fromSocket: Socket,
                toSocket: Socket,
            }
        }
    },
    RemoveNode: {
        data: {
            index: GlobalNodeIndex
        }
    },
    ChangeNodeProperties: {
        data: {
            index: GlobalNodeIndex,
            props: {
                [key: string]: Property
            }
        }
    },
    ChangeNodeUiData: {
        data: {
            index: GlobalNodeIndex,
            uiData: {
                [key: string]: any
            },
        }
    },
    ChangeNodeOverrides: {
        data: {
            index: GlobalNodeIndex,
            overrides: Array<NodeRow>
        }
    },
    ChangeRouteRules: {
        data: {
            newRules: IoRoutes
        }
    }
}>;