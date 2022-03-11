import {createEnumDefinition, EnumInstance} from "../util/enum";
import { GenerationalNode, NodeWrapper } from "./node";

// import {Node, NodeIndex, GenerationalNode} from "./node";

export const PossibleNode = createEnumDefinition({
    "Some": "object", // GenerationalNode
    "None": "number", // generation last held (u32)
});

export class Graph {
    nodes: EnumInstance[]; // PossibleNode

    constructor () {
        this.nodes = [/* PossibleNode {
            Some(GenerationalNode {
                node: NodeWrapper {
                    Node {

                    },
                    NodeIndex {
                        index: usize,
                        generation: u32
                    }
                },
                generation: u32
            }),
            None(u32)
        } */];
    }

    applyJson (json: any) {
        for (let i = 0; i < json.nodes.length; i++) {
            let node = json.nodes[i];

            var index = node.index;

            if (this.nodes[index]) {
                // are they the same generation?

                if (index.generation) {

                }
            }
        }
    }

    getKeyedNodes (): ([string, NodeWrapper])[] {
        let keyedNodes = [];

        for (let i = 0; i < this.nodes.length; i++) {
            this.nodes[i].match([
                [PossibleNode.ids.Some, ([generationalNode]) => {
                    const generation = generationalNode.generation;
                    const nodeWrapper = generationalNode.node;

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

