import { createEnumDefinition, EnumInstance } from "../util/enum";

export const PropertyType = createEnumDefinition({
    "String": null,
    "Integer": null,
    "Float": null,
    "Bool": null,
    "MultipleChoice": "array"
});

PropertyType.deserialize = function (json) {
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
};

export const Property = createEnumDefinition({
    "String": "string",
    "Integer": "number",
    "Float": "number",
    "Bool": "boolean",
    "MultipleChoice": "string"
});

Property.deserialize = function (json) {
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
};

export function jsonToProperty (json: object): EnumInstance {
    return Property[json["type"]](json["content"]);
}
