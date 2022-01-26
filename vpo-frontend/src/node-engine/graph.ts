import {createEnumDefinition, EnumInstance} from "../util/enum";

// import {Node, NodeIndex, GenerationalNode} from "./node";

export const PossibleNode = createEnumDefinition({
    "Some": "object", // GenerationalNode
    "None": "u32", // generation last held
});

function create_new_node (node: Node): EnumInstance {
    return PossibleNode.Some([
        /*GenerationalNode*/ {
            node: /*NodeWrapper*/ {
                node: node,
                index: /*NodeIndex*/ {
                    index: 0,
                    generation: 0,

                },
                connected_inputs: [],
                connected_outputs: []
            },
            generation: 0
        }
    ]);
}

export class Graph {
    nodes: object[]; // PossibleNode

    constructor () {
        this.nodes = [];
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

