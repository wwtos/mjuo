import { DiscriminatedUnion, match } from "../util/discriminated-union"

export interface Index {
    index: number,
    generation: number
}

export const Index = {
    toKey(index: Index): string {
        return index.index + "," + index.generation;
    },
    toString(index: Index): string {
        return `Index { index: ${index.index}, generation: ${index.generation} }`;
    }
};

export type Element<T> = DiscriminatedUnion<"variant", {
    "Occupied": { data: [T, number] },
    "Open": { data: number }
}>;

export const Element = {
    asSome<T>(element: Element<T>): T | null {
        return match(element, {
            Occupied: ({data: [value, _]}) => value,
            Open: () => null
        });
    },
    generation<T>(element: Element<T>): number {
        return match(element, {
            Occupied: ({data: [_, generation]}) => generation,
            Open: ({data: generation}) => generation
        });
    }
};

export interface GenVec<T> {
    vec: Array<Element<T>>
}

export const GenVec = {
    get<T>(genVec: GenVec<T>, index: Index): T | undefined {
        return match(genVec.vec[index.index], {
            Occupied: ({data: [value, generation]}) => {
                if (generation == index.generation) {
                    return value;
                } else {
                    return undefined;
                }
            },
            Open: (_) => undefined,
        });
    }
};