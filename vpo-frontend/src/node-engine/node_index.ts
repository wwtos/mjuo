export interface NodeIndex {
    index: number;
    generation: number;
}

export const NodeIndex = {
    toKey(index: NodeIndex): string {
        return index.index + "," + index.generation;
    },
    toString(index: NodeIndex): string {
        return `NodeIndex { index: ${index.index}, generation: ${index.generation} }`;
    }
};
