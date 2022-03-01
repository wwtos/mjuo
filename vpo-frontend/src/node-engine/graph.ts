import {createEnumDefinition, EnumInstance} from "../util/enum";
import { GenerationalNode, NodeWrapper } from "./node";

// import {Node, NodeIndex, GenerationalNode} from "./node";

export const PossibleNode = createEnumDefinition({
    "Some": "object", // GenerationalNode
    "None": "u32", // generation last held
});

export class Graph {
    nodes: EnumInstance[]; // PossibleNode

    constructor () {
        this.nodes = [];
    }

    getKeyedNodes (): ([string, NodeWrapper])[] {
        let keyedNodes = [];

        for (let i = 0; i < this.nodes.length; i++) {
            this.nodes[i].match([
                [PossibleNode.ids.Some, ([generationalNode]) => {
                    const generation = generationalNode.generation;
                    const nodeWrapper = generationalNode.nodeWrapper;

                    keyedNodes.push([i + "," + generation, nodeWrapper]);
                }]
            ]);
        }

        return keyedNodes;
    }

    // add_node (node: Node): NodeIndex {
    //     let index;
    //     let new_generation;

    //     if (this.nodes.length === 0) {
    //         this.nodes.push(create_new_node(node));

    //         index = this.nodes.length - 1;
    //         new_generation = 0;
    //     } else {
    //         // find an empty slot (if any)
    //     }
    // }
}

