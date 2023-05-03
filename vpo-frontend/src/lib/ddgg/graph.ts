import { match } from "../util/discriminated-union";
import { GenVec, Index } from "./gen_vec";

export type VertexIndex = Index;
export type EdgeIndex = Index;

export interface Vertex<T> {
    connectionsFrom: Array<[VertexIndex, EdgeIndex]>,
    connectionsTo: Array<[VertexIndex, EdgeIndex]>,
    data: T,
}

export interface Edge<T> {
    from: VertexIndex,
    to: VertexIndex,
    data: T
}

export interface Graph<V, E> {
    verticies: GenVec<Vertex<V>>,
    edges: GenVec<Edge<E>>
}

export const Graph = {
    getVertex<V, E>(graph: Graph<V, E>, index: VertexIndex): Vertex<V> | undefined {
        return GenVec.get(graph.verticies, index);
    },
    getVertexData<V, E>(graph: Graph<V, E>, index: VertexIndex): V | undefined {
        return GenVec.get(graph.verticies, index)?.data;
    },
    getEdge<V, E>(graph: Graph<V, E>, index: EdgeIndex): Edge<E> | undefined {
        return GenVec.get(graph.edges, index);
    },
    getEdgeData<V, E>(graph: Graph<V, E>, index: EdgeIndex): E | undefined {
        return GenVec.get(graph.edges, index)?.data;
    },
    verticies<V, E>(graph: Graph<V, E>): Array<[Vertex<V>, Index]> {
        let out: Array<[Vertex<V>, Index]> = [];

        for (let i = 0; i < graph.verticies.length; i++) {
            let elem = graph.verticies[i];

            match(elem, {
                Occupied({data: [vertex, generation]}) {
                    out.push([vertex, {index: i, generation}]);
                },
                Open: (_) => {}
            });
        }

        return out;
    },
    edges<V, E>(graph: Graph<V, E>): Array<[Edge<E>, EdgeIndex]> {
        let out: Array<[Edge<E>, Index]> = [];

        for (let i = 0; i < graph.edges.length; i++) {
            let elem = graph.edges[i];

            match(elem, {
                Occupied({data: [edges, generation]}) {
                    out.push([edges, {index: i, generation}]);
                },
                Open: (_) => {}
            });
        }

        return out;
    }
};