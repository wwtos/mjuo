import { makeTaggedUnion, MemberType, none } from "safety-match";

export const PropertyType = makeTaggedUnion({
    "String": none,
    "Integer": none,
    "Float": none,
    "Bool": none,
    "MultipleChoice": (stringArr: string[]) => stringArr
});

export function deserializePropertyType(json: any): MemberType<typeof PropertyType> {
    switch (json.type) {
        case "String":
            return PropertyType.String;
        case "Integer":
            return PropertyType.Integer;
        case "Float":
            return PropertyType.Float;
        case "Bool":
            return PropertyType.Bool;
        case "MultipleChoice":
            return PropertyType.MultipleChoice(json.content);
    }

    throw "Failed to parse json";    
};

export const Property = makeTaggedUnion({
    "String": (string: string) => string,
    "Integer": (integer: number) => integer,
    "Float": (float: number) => float,
    "Bool": (boolean: boolean) => boolean,
    "MultipleChoice": (choice: string) => choice
});

export function deserializeProperty(json: any): MemberType<typeof Property> {
    switch (json.type) {
        case "String":
            return Property.String(json.content);
        case "Integer":
            return Property.Integer(json.content);
        case "Float":
            return Property.Float(json.content);
        case "Bool":
            return Property.Bool(json.content);
        case "MultipleChoice":
            return Property.MultipleChoice(json.content);
    }

    throw "Failed to parse json";   
};

export function jsonToProperty (json: object): MemberType<typeof Property> {
    return Property[json["type"]](json["content"]);
}
