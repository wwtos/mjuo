import { NodeGraph } from '../node-engine/node_graph';
import { NodeIndex } from "../node-engine/node";

export function deselectAll (graph: NodeGraph): NodeIndex[] {
    const currentNodes = graph.nodeStore.getValue();

    let touchedNodes: NodeIndex[] = [];

    for (let currentNode of currentNodes) {
        if (!currentNode) continue;

        if (currentNode.uiData.getValue().selected) {
            touchedNodes.push(currentNode.index);

            currentNode.uiData.next({
                ...currentNode.uiData.getValue(),
                selected: false
            });
        }
    }

    return touchedNodes;
}
