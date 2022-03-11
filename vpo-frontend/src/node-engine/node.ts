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

    constructor(
        node: Node,
        index: NodeIndex,
        connectedInputs: EnumInstance[]/*[InputSideConnection]*/,
        connectedOutputs: EnumInstance[]/*[OutputSideConnection]*/
    ) {
        this.node = node;
        this.index = index;
        this.connectedInputs = connectedInputs;
        this.connectedOutputs = connectedOutputs;
    }
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
