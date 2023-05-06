import type { VertexIndex } from "$lib/ddgg/graph";
import type { NodeGraph } from "$lib/node-engine/node_graph";

export function deselectAll(graph: NodeGraph): VertexIndex[] {
    const currentNodes = graph.nodeStore.getValue();

    let touchedNodes = currentNodes.filter(([node, index]) => {
        return node.data.uiData.selected;
    });

    for (let [node, index] of touchedNodes) {
        node.data.uiData = {
            ...node.data.uiData,
            selected: false,
        };

        graph.updateNode(index);
    }

    return touchedNodes.map(([_, index]) => index);
}

export function preventHistoryKeyActions(
    this: HTMLInputElement,
    event: KeyboardEvent
) {
    if (event.ctrlKey) {
        if (event.key === "z" || event.key === "y") event.preventDefault();
    } else {
        event.stopPropagation();
    }
}