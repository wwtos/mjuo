import { createEnumDefinition, EnumInstance } from "../util/enum";
import { InputSideConnection, MidiSocketType, OutputSideConnection, Primitive, SocketType, StreamSocketType, ValueSocketType } from "./connection";
import { Property, PropertyType } from "./property";
import { Readable, readable } from "svelte/store";

export const NodeRow = createEnumDefinition({
    "StreamInput": [StreamSocketType, "f32"],
    "MidiInput": [MidiSocketType, "array"],
    "ValueInput": [ValueSocketType, Primitive],
    "StreamOutput": [StreamSocketType, "f32"],
    "MidiOutput": [MidiSocketType, "array"],
    "ValueOutput": [ValueSocketType, Primitive],
    "Property": ["string", PropertyType, Property]
});

NodeRow.deserialize = function (json) {
    console.log("deserializing", json);
    switch (json.type) {
        case "StreamInput":
            return NodeRow.StreamInput(StreamSocketType.deserialize(json.content[0]), json.content[1]);
        case "MidiInput":
            return NodeRow.MidiInput(MidiSocketType.deserialize(json.content[0]), json.content[1]);
        case "ValueInput":
            return NodeRow.ValueInput(ValueSocketType.deserialize(json.content[0]), Primitive.deserialize(json.content[1]));
        case "StreamOutput":
            return NodeRow.StreamOutput(StreamSocketType.deserialize(json.content[0]), json.content[1]);
        case "MidiOutput":
            return NodeRow.MidiOutput(MidiSocketType.deserialize(json.content[0]), json.content[1]);
        case "ValueOutput":
            return NodeRow.ValueOutput(ValueSocketType.deserialize(json.content[0]), Primitive.deserialize(json.content[1]));
        case "Property":
            return NodeRow.Property(json.content[0], PropertyType.deserialize(json.content[1]), Property.deserialize(json.content[2]));
    }
};

export class InitResult {
    did_rows_change: boolean;
    /* Vec<NodeRow> */
    node_rows: Array<EnumInstance>;
    /* Option<HashMap<String, Property>> */
    changed_properties?: {[key: string]: EnumInstance}

    constructor (obj) {
        for (var i in obj) {
            this[i] = obj[i];
        }
    }
}

export class UiData {
    x?: number = 0;
    y?: number = 0;
    selected?: boolean = false;
    title?: string = "Node";

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
        [prop: string]: EnumInstance // HashMap<String, PropertyType>
    }; // hashmap of property/propertyType pairs

    constructor(
        inputSockets: EnumInstance[],
        outputSockets: EnumInstance[],
        usableProperties: { [prop: string]: EnumInstance /*HashMap<String, PropertyType>*/ }
    ) {
        this.inputSockets = inputSockets;
        this.outputSockets = outputSockets;
        this.usableProperties = usableProperties;
    }
}

export class NodeWrapper {
    private node: Readable<Node>;
    private index: Readable<NodeIndex>;
    private connectedInputs: Readable<InputSideConnection[]>;
    private connectedOutputs: Readable<OutputSideConnection[]>;
    private nodeRows: /* NodeRow */Readable<EnumInstance[]>;
    private properties: Readable<object>;
    private uiData: Readable<UiData>;

    constructor(
        node: Node,
        index: NodeIndex,
        connectedInputs: InputSideConnection[],
        connectedOutputs: OutputSideConnection[],
        nodeRows: /* NodeRow */EnumInstance[],
        properties: object,
        uiData: UiData
    ) {
        const self = this;

        this.node = readable(node, function start(set) {
            self.updateNode = set;
        });

        this.index = readable(index, function start(set) {
            self.updateIndex = set;
        });

        this.connectedInputs = readable(connectedInputs, function start(set) {
            self.updateConnectedInputs = set;
        });

        this.connectedOutputs = readable(connectedOutputs, function start(set) {
            self.updateConnectedOutputs = set;
        });

        this.nodeRows = readable(nodeRows, function start(set) {
            self.updateNodeRows = set;
        });

        this.properties = readable(properties, function start(set) {
            self.updateProperties = set;
        });

        this.uiData = readable(uiData, function start(set) {
            
        });
    }

    updateNode(node: Node) {}

    updateIndex(index: NodeIndex) {}

    updateConnectedInputs(connectedInputs: InputSideConnection[]) {}

    updateConnectedOutputs(connectedOutputs: OutputSideConnection[]) {}

    updateNodeRows(nodeRows: /* NodeRow */EnumInstance[]) {}

    updateProperties(properties: object) {}

    updateUiData(uiData: UiData) {}

    toJSON(): object {
        return {
            index: this.index,
            connected_inputs: this.connectedInputs,
            connected_outputs: this.connectedOutputs,
            properties: this.properties,
            ui_data: this.uiData
        };
    }

    getInputConnectionByType(inputSocketType: EnumInstance /* SocketType */): Readable<InputSideConnection | undefined> {
        return this.connectedInputs.find(input => input.toSocketType === inputSocketType);
    }

    getOutputConnectionsByType(outputSocketType: EnumInstance /* SocketType */): OutputSideConnection[] {
        return this.connectedOutputs.filter(input => input.fromSocketType === outputSocketType);
    }

    // list_input_sockets(&self) => EnumInstance[] /* Vec<SocketType> */ {
    //     this.nodeRows.map(row => {
    //         row.match(
    //         [NodeRow.ids.StreamInput, streamInputType => SocketType.Stream(streamInputType)],
    //         [NodeRow.ids.MidiInput, midiInputType => SocketType.Midi(midiInputType)]),
    //         [NodeRow.ids.MidiInput, midiInputType => SocketType.Midi(midiInputType)]),
    //     })

    //     this.node_rows.filter_map(|row| {
    //             NodeRow::MidiInput(midi_input_type, _) => Some(SocketType::Midi(midi_input_type.clone())),
    //             NodeRow::ValueInput(value_input_type, _) => Some(SocketType::Value(value_input_type.clone())),
    //             NodeRow::StreamOutput(_, _) => None,
    //             NodeRow::MidiOutput(_, _) => None,
    //             NodeRow::ValueOutput(_, _) => None,
    //             NodeRow::Property(..) => None,
    //         }
    //     }).collect()
    // }

    // pub fn list_output_sockets(&self) -> Vec<SocketType> {
    //     self.node_rows.iter().filter_map(|row| {
    //         match row {
    //             NodeRow::StreamInput(_, _) => None,
    //             NodeRow::MidiInput(_, _) => None,
    //             NodeRow::ValueInput(_, _) => None,
    //             NodeRow::StreamOutput(stream_output_type, _) => Some(SocketType::Stream(stream_output_type.clone())),
    //             NodeRow::MidiOutput(midi_output_type, _) => Some(SocketType::Midi(midi_output_type.clone())),
    //             NodeRow::ValueOutput(value_output_type, _) => Some(SocketType::Value(value_output_type.clone())),
    //             NodeRow::Property(..) => None,
    //         }
    //     }).collect()
    // }
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
