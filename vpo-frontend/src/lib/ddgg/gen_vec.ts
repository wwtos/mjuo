import { type DiscriminatedUnion, match } from "../util/discriminated-union";

export interface Index {
    index: number,
    generation: number
}

export const Index = {
    fromString(index: string): Index {
        let [arrIndex, generation] = index.split(".");

        return {
            index: parseInt(arrIndex),
            generation: parseInt(generation)
        };
    },
    toString(index: Index): string {
        return `${index.index}.${index.generation}`;
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

export type GenVec<T> = Array<Element<T>>;

export const GenVec = {
    get<T>(genVec: GenVec<T>, strIndex: string): T | undefined {
        let index = Index.fromString(strIndex);
        
        if (!genVec[index.index]) return undefined;

        return match(genVec[index.index], {
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