import type { DiscriminatedUnion } from "../util/discriminated-union";

export type PropertyType = DiscriminatedUnion<"variant", {
    String: {},
    Integer: {},
    Float: {},
    Bool: {},
    MultipleChoice: { data: string[] },
    Resource: { data: string }
}>;

export type Property = DiscriminatedUnion<"variant", {
    String: { data: string },
    Integer: { data: number },
    Bool: { data: boolean },
    MultipleChoice: { data: string },
    Resource: { data: { namespace: string, resource: string } }
}>;
