import {createEnumDefinition} from "../util/enum";

export const PossibleNode = createEnumDefinition({
    "Some": "object", // GenerationalNode
    "None": "u32", // generation last held
})

export interface Graph {
    nodes: object[] // PossibleNode
}

