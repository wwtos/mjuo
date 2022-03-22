import { EnumInstance } from "../util/enum";
import { InputSideConnection, OutputSideConnection } from "./connection";

export class UIData {
    x: number = 0;
    y: number = 0;
    selected: boolean = false;
    title: string = "Node";

    constructor(props: object) {
        for (var prop in props) {
            this[prop] = props[prop];
        }
    }
}

export class Node {
    inputSockets: EnumInstance[]; // Vec<SocketType>
    outputSockets: EnumInstance[]; // Vec<SocketType>
    usableProperties: {
        [prop: string]: EnumInstance[] // HashMap<String, PropertyType>
    }; // hashmap of property/propertyType pairs

    constructor(
        inputSockets: EnumInstance[],
        outputSockets: EnumInstance[],
        usableProperties: { [prop: string]: EnumInstance[] /*HashMap<String, PropertyType>*/ }
    ) {
        this.inputSockets = inputSockets;
        this.outputSockets = outputSockets;
        this.usableProperties = usableProperties;
    }
}

export class NodeWrapper {
    node: Node;
    index: NodeIndex;
    connectedInputs: InputSideConnection[];
    connectedOutputs: OutputSideConnection[];
    properties: object;
    uiData: UIData;

    constructor(
        node: Node,
        index: NodeIndex,
        connectedInputs: InputSideConnection[],
        connectedOutputs: OutputSideConnection[],
        properties: object,
        uiData: UIData
    ) {
        this.node = node;
        this.index = index;
        this.connectedInputs = connectedInputs;
        this.connectedOutputs = connectedOutputs;
        this.properties = properties;
        this.uiData = uiData;
    }

    toJSON(): object {
        return {
            index: this.index,
            connected_inputs: this.connectedInputs,
            connected_outputs: this.connectedOutputs,
            properties: this.properties,
            ui_data: this.uiData
        };
    }
}

export class NodeIndex {
    index: number;
    generation: number;

    constructor(index: number, generation: number) {
        this.index = index;
        this.generation = generation;
    }

    toKey(): string {
        return this.index + "," + this.generation;
    }
}

export interface GenerationalNode {
    node: NodeWrapper,
    generation: number
}
