import { EnumInstance } from "../util/enum";

export class Node {
    inputSockets: EnumInstance[]; // Vec<SocketType>
    outputSockets: EnumInstance[]; // Vec<SocketType>
    listProperties: () => object; // hashmap of property/propertyType pairs
    serializeToJson:() => object;
    applyJson: (json: object) => void;
}

export class NodeWrapper {
    node: Node;
    index: NodeIndex;
    /** [InputSideConnection] */
    connectedInputs: EnumInstance[];
    /** [OutputSideConnection] */
    connectedOutputs: EnumInstance[];
}

export class NodeIndex {
    index: number;
    generation: number;

    constructor(index: number, generation: number) {
        this.index = index;
        this.generation = generation;
    }

    toString(): string {
        return this.index + "," + this.generation;
    }
}

export interface GenerationalNode {
    node: NodeWrapper,
    generation: number
}
