import { createEnumDefinition } from "../util/enum";

export const PropertyType = createEnumDefinition({
    "String": "string",
    "Integer": "number",
    "Float": "number",
    "Bool": "boolean"
});
