import { createEnumDefinition, EnumInstance } from "../util/enum";

export const PropertyType = createEnumDefinition({
    "String": null,
    "Integer": null,
    "Float": null,
    "Bool": null,
    "MultipleChoice": "array"
});

export const Property = createEnumDefinition({
    "String": "string",
    "Integer": "number",
    "Float": "number",
    "Bool": "boolean",
    "MultipleChoice": "string"
});

export function jsonToProperty (json: object): EnumInstance {
    return Property[json["type"]](json["content"]);
}
