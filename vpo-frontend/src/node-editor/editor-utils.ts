import { NodeGraph } from '../node-engine/node_graph';
import type { NodeIndex } from "../node-engine/node_index";

export function deselectAll (graph: NodeGraph): NodeIndex[] {
    const currentNodes = graph.nodeStore.getValue();

    let touchedNodes: NodeIndex[] = [];

    for (let currentNode of currentNodes) {
        if (!currentNode) continue;

        if (currentNode.ui_data.selected) {
            touchedNodes.push(currentNode.index);

            currentNode.ui_data = {
                ...currentNode.ui_data,
                selected: false
            };

            graph.updateNode(currentNode.index);
        }
    }

    return touchedNodes;
}
