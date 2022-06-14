import { createEnumDefinition, EnumInstance } from "../util/enum";
import { InputSideConnection, MidiSocketType, OutputSideConnection, Primitive, SocketDirection, SocketType, StreamSocketType, ValueSocketType } from "./connection";
import { Property, PropertyType } from "./property";
import { BehaviorSubject, Observable, Subject } from "rxjs";
import { distinctUntilChanged, map, mergeMap } from "rxjs/operators";
import { shallowEqual } from 'fast-equals';
import { Readable, writable, Writable } from "svelte/store";
import { wrapStore } from "../util/wrap-store";

const TITLE_HEIGHT = 30;
const SOCKET_HEIGHT = 36;
const SOCKET_OFFSET = 24;
const NODE_WIDTH = 200;

export const NodeRow = createEnumDefinition({
    "StreamInput": [StreamSocketType, "f32"],
    "MidiInput": [MidiSocketType, "array"],
    "ValueInput": [ValueSocketType, Primitive],
    "StreamOutput": [StreamSocketType, "f32"],
    "MidiOutput": [MidiSocketType, "array"],
    "ValueOutput": [ValueSocketType, Primitive],
    "Property": ["string", PropertyType, Property]
});

NodeRow.asTypeAndDirection = function (nodeRow: EnumInstance): [EnumInstance/* SocketType */, SocketDirection] {
    return nodeRow.match([
        [NodeRow.ids.StreamInput, ([socketType]) => [SocketType.Stream(socketType), SocketDirection.Input]],
        [NodeRow.ids.MidiInput, ([socketType]) => [SocketType.Midi(socketType), SocketDirection.Input]],
        [NodeRow.ids.ValueInput, ([socketType]) => [SocketType.Value(socketType), SocketDirection.Input]],
        [NodeRow.ids.StreamOutput, ([socketType]) => [SocketType.Stream(socketType), SocketDirection.Output]],
        [NodeRow.ids.MidiOutput, ([socketType]) => [SocketType.Midi(socketType), SocketDirection.Output]],
        [NodeRow.ids.ValueOutput, ([socketType]) => [SocketType.Value(socketType), SocketDirection.Output]],
    ]);
};

NodeRow.deserialize = function (json) {
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
    node: BehaviorSubject<Node>;
    index: NodeIndex;
    connectedInputs: BehaviorSubject<InputSideConnection[]>;
    connectedOutputs: BehaviorSubject<OutputSideConnection[]>;
    nodeRows: /* NodeRow */BehaviorSubject<EnumInstance[]>;
    properties: BehaviorSubject<object>;
    uiData: BehaviorSubject<UiData>;

    constructor(
        node: Node,
        index: NodeIndex,
        connectedInputs: InputSideConnection[],
        connectedOutputs: OutputSideConnection[],
        nodeRows: /* NodeRow */EnumInstance[],
        properties: object,
        uiData: UiData
    ) {
        this.node = new BehaviorSubject(node);
        this.index = index;
        this.connectedInputs = new BehaviorSubject(connectedInputs);
        this.connectedOutputs = new BehaviorSubject(connectedOutputs);
        this.nodeRows = new BehaviorSubject(nodeRows);
        this.properties = new BehaviorSubject(properties);
        this.uiData = new BehaviorSubject(uiData);
    }

    toJSON(): object {
        return {
            index: this.index,
            connected_inputs: this.connectedInputs.getValue(),
            connected_outputs: this.connectedOutputs.getValue(),
            properties: this.properties.getValue(),
            ui_data: this.uiData.getValue()
        };
    }

    getInputConnectionByType(inputSocketType: EnumInstance /* SocketType */): Observable<InputSideConnection | undefined> {
        return this.connectedInputs.pipe(
            map(connectedInputs => {
                return connectedInputs.find(input => input.toSocketType === inputSocketType);
            }),
            distinctUntilChanged(shallowEqual)
        );
    }

    getOutputConnectionsByType(outputSocketType: EnumInstance /* SocketType */): Observable<OutputSideConnection[]> {
        return this.connectedOutputs.pipe(
            map(connectedOutputs => {
                return connectedOutputs.filter(input => input.fromSocketType === outputSocketType);
            },
            distinctUntilChanged(shallowEqual)
        ));
    }

    getSocketXY(socketType: EnumInstance /* SocketType */, direction: SocketDirection): Observable<[number, number] | undefined> {
        return this.nodeRows.pipe(
            mergeMap(nodeRows => {
                const rowIndex = nodeRows.findIndex(nodeRow => {
                    const [rowSocketType, rowDirection] = NodeRow.asTypeAndDirection(nodeRow);

                    return socketType.getType() === rowSocketType.getType() &&
                           (socketType.content as any).getType() === rowSocketType.content.getType() &&
                           direction === rowDirection;
                });

                if (rowIndex === -1) return undefined;

                const relativeX = direction === SocketDirection.Output ? NODE_WIDTH : 0;
                const relativeY = TITLE_HEIGHT + rowIndex * SOCKET_HEIGHT + SOCKET_OFFSET;

                return this.uiData.pipe<[number, number]>(
                    map(uiData => [uiData.x + relativeX, uiData.y + relativeY])
                );
            })
        );
    }

    getSocketXYCurrent(socketType: EnumInstance /* SocketType */, direction: SocketDirection): [number, number] | undefined {
        const nodeRows = this.nodeRows.getValue();
        const rowIndex = nodeRows.findIndex(nodeRow => {
            const [rowSocketType, rowDirection] = NodeRow.asTypeAndDirection(nodeRow);

            return socketType.getType() === rowSocketType.getType() &&
                    (socketType.content[0] as any).getType() === rowSocketType.content[0].getType() &&
                    direction === rowDirection;
        });

        if (rowIndex === -1) return undefined;

        const relativeX = direction === SocketDirection.Output ? NODE_WIDTH : 0;
        const relativeY = TITLE_HEIGHT + rowIndex * SOCKET_HEIGHT + SOCKET_OFFSET;

        const uiData = this.uiData.getValue();

        return [uiData.x + relativeX, uiData.y + relativeY];
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
