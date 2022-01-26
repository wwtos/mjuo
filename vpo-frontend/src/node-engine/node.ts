export interface Node {
    listInputSockets: () => object[], // Vec<SocketType>
    listOutputSockets: () => object[], // Vec<SocketType>
    listProperties: () => object, // hashmap of property/value pairs
    serializeToJson:() => object,
    deserializeFromJson: () => Node
}

export interface NodeWrapper {
    node: Node,
    index: NodeIndex,
    /** [InputSideConnection] */
    connected_inputs: [object],
    /** [OutputSideConnection] */
    connected_outputs: [object]
}

export interface NodeIndex {
    index: number,
    generation: number
}

export interface GenerationalNode {
    node: NodeWrapper,
    generation: number
}
