import { areSocketTypesEqual, InputSideConnection, MidiSocketType, NodeRefSocketType, OutputSideConnection,
         Primitive, SocketDirection, SocketType, StreamSocketType, ValueSocketType, deserializeStreamSocketType,
         deserializeMidiSocketType, deserializeNodeRefSocketType, deserializeValueSocketType, deserializePrimitive } from "./connection";
import { deserializeProperty, deserializePropertyType, Property, PropertyType } from "./property";
import { BehaviorSubject, combineLatest, Observable } from "rxjs";
import { distinctUntilChanged, map, mergeMap } from "rxjs/operators";
import { shallowEqual } from 'fast-equals';
import { makeTaggedUnion, MemberType } from "safety-match";

const TITLE_HEIGHT = 30;
const SOCKET_HEIGHT = 36;
const SOCKET_OFFSET = 26;
const NODE_WIDTH = 200;

export const NodeRow = makeTaggedUnion({
    "StreamInput": (type: MemberType<typeof StreamSocketType>, defaultVal: number): [MemberType<typeof StreamSocketType>, number] => [type, defaultVal],
    "MidiInput": (type: MemberType<typeof MidiSocketType>, defaultVal: any[]): [MemberType<typeof MidiSocketType>, any[]] => [type, defaultVal],
    "ValueInput": (type: MemberType<typeof ValueSocketType>, defaultVal: MemberType<typeof Primitive>): [MemberType<typeof ValueSocketType>, MemberType<typeof Primitive>] => [type, defaultVal],
    "NodeRefInput": (type: MemberType<typeof NodeRefSocketType>) => type,
    "StreamOutput": (type: MemberType<typeof StreamSocketType>, defaultVal: number): [MemberType<typeof StreamSocketType>, number] => [type, defaultVal],
    "MidiOutput": (type: MemberType<typeof MidiSocketType>, defaultVal: any[]): [MemberType<typeof MidiSocketType>, any[]] => [type, defaultVal],
    "ValueOutput": (type: MemberType<typeof ValueSocketType>, defaultVal: MemberType<typeof Primitive>): [MemberType<typeof ValueSocketType>, MemberType<typeof Primitive>] => [type, defaultVal],
    "NodeRefOutput": (type: MemberType<typeof NodeRefSocketType>) => type,
    "Property": (name: string, type: MemberType<typeof PropertyType>, defaultVal: MemberType<typeof Property>): [string, MemberType<typeof PropertyType>, MemberType<typeof Property>] => [name, type, defaultVal]
});

const NodeRowAsTypeAndDirection = function (nodeRow: MemberType<typeof NodeRow > ): /* [MemberType<typeof SocketType>, SocketDirection]*/ any {
    return nodeRow.match({
        StreamInput: ([type, _]) => [SocketType.Stream(type), SocketDirection.Input],
        MidiInput: ([type, _]) => [SocketType.Midi(type), SocketDirection.Input],
        ValueInput: ([type, _]) => [SocketType.Value(type), SocketDirection.Input],
        NodeRefInput: (type) => [SocketType.NodeRef(type), SocketDirection.Input],
        StreamOutput: ([type, _]) => [SocketType.Stream(type), SocketDirection.Output],
        MidiOutput: ([type, _]) => [SocketType.Midi(type), SocketDirection.Output],
        ValueOutput: ([type, _]) => [SocketType.Value(type), SocketDirection.Output],
        NodeRefOutput: (type) => [SocketType.NodeRef(type), SocketDirection.Output],
        Property: ([name, type, defaultVal]) => [SocketType.Stream(StreamSocketType.Audio), SocketDirection.Input]
    });
};

const NodeRowFromTypeAndDirection = function (type: MemberType<typeof SocketType>, direction: SocketDirection, defaultValue: any): MemberType<typeof NodeRow> {
    if (direction === SocketDirection.Input) {
        return type.match({
            Stream: (streamSocketType) => NodeRow.StreamInput(streamSocketType, defaultValue),
            Midi: (midiSocketType) => NodeRow.MidiInput(midiSocketType, defaultValue),
            Value: (valueSocketType) => NodeRow.ValueInput(valueSocketType, defaultValue),
            NodeRef: (nodeRefSocketType) => NodeRow.NodeRefInput(nodeRefSocketType),
            MethodCall: (params) => {
                throw "why do I still have this"
            }
        });
    } else {
        return type.match({
            Stream: (streamSocketType) => NodeRow.StreamOutput(streamSocketType, defaultValue),
            Midi: (midiSocketType) => NodeRow.MidiOutput(midiSocketType, defaultValue),
            Value: (valueSocketType) => NodeRow.ValueOutput(valueSocketType, defaultValue),
            NodeRef: (nodeRefSocketType) => NodeRow.NodeRefOutput(nodeRefSocketType),
            MethodCall: (params) => {
                throw "why do I still have this"
            }
        });
    }
};

const deserializeNodeRow = function (json) {
    switch (json.type) {
        case "StreamInput":
            return NodeRow.StreamInput(deserializeStreamSocketType(json.content[0]), json.content[1]);
        case "MidiInput":
            return NodeRow.MidiInput(deserializeMidiSocketType(json.content[0]), json.content[1]);
        case "ValueInput":
            return NodeRow.ValueInput(deserializeValueSocketType(json.content[0]), deserializePrimitive(json.content[1]));
        case "NodeRefInput":
            return NodeRow.NodeRefInput(deserializeNodeRefSocketType(json.content[0]));
        case "StreamOutput":
            return NodeRow.StreamOutput(deserializeStreamSocketType(json.content[0]), json.content[1]);
        case "MidiOutput":
            return NodeRow.MidiOutput(deserializeMidiSocketType(json.content[0]), json.content[1]);
        case "ValueOutput":
            return NodeRow.ValueOutput(deserializeValueSocketType(json.content[0]), deserializePrimitive(json.content[1]));
        case "NodeRefOutput":
            return NodeRow.NodeRefOutput(deserializeNodeRefSocketType(json.content[0]));
        case "Property":
            return NodeRow.Property(json.content[0], deserializePropertyType(json.content[1]), deserializeProperty(json.content[2]));
    }
};


export class InitResult {
    did_rows_change: boolean;
    /* Vec<NodeRow> */
    node_rows: Array <MemberType<typeof NodeRow>> ;
    /* Option<HashMap<String, Property>> */
    changed_properties ? : {
        [key: string]: MemberType<typeof Property>
    }

    constructor(obj) {
        for (var i in obj) {
            this[i] = obj[i];
        }
    }
}

export class UiData {
    x: number = 0;
    y: number = 0;
    selected ? : boolean = false;
    title ? : string = "Node";

    constructor(props: object) {
        for (var prop in props) {
            this[prop] = props[prop];
        }
    }
}

export class Node {
    inputSockets: MemberType<typeof SocketType>[];
    outputSockets: MemberType<typeof SocketType>[];
    usableProperties: {
        [prop: string]: MemberType<typeof PropertyType>
    }; 

    constructor(
        inputSockets: MemberType<typeof SocketType>[],
        outputSockets: MemberType<typeof SocketType>[],
        usableProperties: {
            [prop: string]: MemberType<typeof PropertyType>
        }
    ) {
        this.inputSockets = inputSockets;
        this.outputSockets = outputSockets;
        this.usableProperties = usableProperties;
    }
}

export class NodeWrapper {
    node: BehaviorSubject<Node>;
    index: NodeIndex;
    defaultOverrides: BehaviorSubject<MemberType<typeof NodeRow>[]>;
    connectedInputs: BehaviorSubject<InputSideConnection[]>;
    connectedOutputs: BehaviorSubject<OutputSideConnection[]>;
    nodeRows: BehaviorSubject<MemberType<typeof NodeRow>[]>;
    properties: BehaviorSubject<object>;
    uiData: BehaviorSubject<UiData>;

    constructor(
        node: Node,
        index: NodeIndex,
        connectedInputs: InputSideConnection[],
        connectedOutputs: OutputSideConnection[],
        defaultOverrides: MemberType<typeof NodeRow>[],
        nodeRows: MemberType<typeof NodeRow>[],
        properties: object,
        uiData: UiData
    ) {
        this.node = new BehaviorSubject(node);
        this.index = index;
        this.connectedInputs = new BehaviorSubject(connectedInputs);
        this.connectedOutputs = new BehaviorSubject(connectedOutputs);
        this.nodeRows = new BehaviorSubject(nodeRows);
        this.defaultOverrides = new BehaviorSubject(defaultOverrides);
        this.properties = new BehaviorSubject(properties);
        this.uiData = new BehaviorSubject(uiData);
    }

    toJSON(): object {
        return {
            index: this.index,
            connected_inputs: this.connectedInputs.getValue(),
            connected_outputs: this.connectedOutputs.getValue(),
            properties: this.properties.getValue(),
            default_overrides: this.defaultOverrides.getValue(),
            ui_data: this.uiData.getValue()
        };
    }

    getInputConnectionByType(inputSocketType: MemberType<typeof SocketType>): Observable < InputSideConnection | undefined > {
        return this.connectedInputs.pipe(
            map(connectedInputs => {
                return connectedInputs.find(input => areSocketTypesEqual(input.toSocketType, inputSocketType));
            }),
            distinctUntilChanged(shallowEqual)
        );
    }

    getOutputConnectionsByType(outputSocketType: MemberType<typeof SocketType>): Observable < OutputSideConnection[] > {
        return this.connectedOutputs.pipe(
            map(connectedOutputs => {
                    return connectedOutputs.filter(input => input.fromSocketType === outputSocketType);
                },
                distinctUntilChanged(shallowEqual)
            ));
    }

    getPropertyValue(propertyName: string): Observable < any > {
        return combineLatest([this.properties, this.nodeRows]).pipe(
            map(([properties, nodeRows]) => {
                if (properties[propertyName] !== undefined) {
                    return properties[propertyName];
                } else {
                    // else find property in defaults
                    const row = nodeRows.find(nodeRow => {
                        return nodeRow.match({
                            Property: ([rowName, rowType, rowDefault]) => {
                                return rowName === propertyName;
                            },
                            _: () => { return false; }
                        });
                    });

                    if (!row) return undefined;

                    return row.match({
                        Property: ([_name, _type, defaultVal]) => defaultVal,
                        _: () => { throw "unreachable" }
                    });
                }
            })
        );
    }

    getSocketDefault(socketType: MemberType<typeof SocketType>, direction: SocketDirection): Observable<any> {
        return combineLatest([this.nodeRows, this.defaultOverrides]).pipe(
            map(([nodeRows, defaultOverrides]) => {
                const defaultOverride = defaultOverrides.find(defaultOverride => {
                    const [overrideSocketType, overrideDirection] = NodeRowAsTypeAndDirection(defaultOverride);

                    return areSocketTypesEqual(socketType, overrideSocketType) &&
                        direction === overrideDirection;
                });

                if (defaultOverride) return (defaultOverride.data)[1];

                const defaultNodeRow = nodeRows.find(nodeRow => {
                    const [nodeRowSocketType, nodeRowDirection] = NodeRowAsTypeAndDirection(nodeRow);

                    return areSocketTypesEqual(socketType, nodeRowSocketType) &&
                        direction === nodeRowDirection;
                });

                if (defaultNodeRow) return (defaultNodeRow.data)[1];
            })
        );
    }

    getSocketXY(socketType: MemberType<typeof SocketType>, direction: SocketDirection): Observable < [number, number] | undefined > {
        return this.nodeRows.pipe(
            mergeMap(nodeRows => {
                const rowIndex = nodeRows.findIndex(nodeRow => {
                    const [rowSocketType, rowDirection] = NodeRowAsTypeAndDirection(nodeRow);

                    return areSocketTypesEqual(socketType, rowSocketType);
                });

                if (rowIndex === -1) return new BehaviorSubject(undefined);

                const relativeX = direction === SocketDirection.Output ? NODE_WIDTH : 0;
                const relativeY = TITLE_HEIGHT + rowIndex * SOCKET_HEIGHT + SOCKET_OFFSET;

                return this.uiData.pipe < [number, number] > (
                    map(uiData => [uiData.x + relativeX, uiData.y + relativeY])
                );
            })
        );
    }

    getSocketXYCurrent(socketType: MemberType<typeof SocketType>, direction: SocketDirection): [number, number] | undefined {
        const nodeRows = this.nodeRows.getValue();
        const rowIndex = nodeRows.findIndex(nodeRow => {
            const [rowSocketType, rowDirection] = NodeRowAsTypeAndDirection(nodeRow);

            return areSocketTypesEqual(socketType, rowSocketType);;
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