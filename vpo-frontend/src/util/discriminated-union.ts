export type DiscriminatedUnion<K extends PropertyKey, T extends object> = {
    [P in keyof T]: ({ [Q in K]: P } & T[P]) extends infer U ? { [Q in keyof U]: U[Q] } : never
}[keyof T];


type Discriminated<V extends string> = { variant: V };
type Narrowed<T extends { variant: string }, V extends string> = T & { variant: V };

export function match<
    Variants extends string,
    Instance extends Discriminated<Variants>,
    ReturnType
>(
    instance: Instance,
    matchArms: { [Variant in Variants]: (a: Narrowed<Instance, Variant>) => ReturnType }
): ReturnType {
    return matchArms[instance.variant](instance);
}

export function matchOrElse<
    Variants extends Instance['variant'],
    Instance extends Discriminated<string>,
    ReturnType
>(
    instance: Instance,
    matchArms: { [Variant in Variants]: (a: Narrowed<Instance, Variant>) => ReturnType },
    orElse: (a: Narrowed<Instance, Exclude<Instance['variant'], Variants>>) => ReturnType
): ReturnType {
    return instance.variant in matchArms
               ? matchArms[instance.variant as Variants](instance as Narrowed<Instance, Variants>)
               : orElse(instance as Narrowed<Instance, Exclude<Instance['variant'], Variants>>);
};
