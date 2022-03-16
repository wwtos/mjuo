export class EnumInstance {
    enumDef: any; // Really an EnumDefinition, except EnumDefinition is such a fluid type
    type: number;
    value: object | null;

    constructor (enumDef: object, type: number, value: object | null) {
        this.enumDef = enumDef;
        this.type = type;
        this.value = value;
    }

    getType(): number {
        return this.type;
    }

    toJSON () {
        let ids = this.enumDef.ids;
        let typeName: string = "";
    
        for (let id of Object.keys(ids)) {
            if (this.type === ids[id]) {
                typeName = id;
            }
        }

        let value = this.value;

        if (Array.isArray(this.enumDef.states[typeName]) && this.enumDef.states[typeName].length === 1) {
            value = value[0];
        }
    
        return {
            "type": typeName,
            "content": value
        };
    }

    /** match clauses is an object
      * for example: 
      * Enum.match([
      *     [Enum.Value, ([x1, y1]) => {
      *         // return {"didThisMatch": false}
      *         // if it didn't match
      *         // otherwise just return the value as normal
      *     }]
      * ]); **/
    match (matchClauses: (Function | number)[][]): any {
        // debugging //
        if (matchClauses.findIndex(clause => clause === undefined) !== -1) {
            throw "One of the match clauses is undefined!";
        }

        // go through in order through the match clauses
        for (var matchClause of matchClauses) {
            // don't even run it if it's not of that type
            var clauseType = matchClause[0];
    
            var returnValue: any;
    
            // -1 is catch all
            if (clauseType === -1) {
                returnValue = (matchClause[1] as Function)(this.value);
            } else if (clauseType === this.type) {
                returnValue = (matchClause[1] as Function)(this.value);
            } else if (clauseType !== this.type) {
                continue;
            }
    
            if (returnValue && returnValue.didThisMatch === false) {
                continue; // check the next one
            }
    
            return returnValue;
        }
    }
}



// checks that `value` conforms to `type`
function verifyInput(type: (string | Function), value: any) {
    if (type instanceof EnumDefinition) {
        return value.enumDef === type;
    } else if (type instanceof Function) {
        // for custom validators
        return type(value);
    } else if (type === "object") {
        return value !== null && typeof value === "object";
    } else if (type === "array") {
        return Array.isArray(value);
    } else if (type === "string") {
        return typeof value === "string";
    } else if (type === "f32" || type === "i32" || type === "u32" || type === "u64" || type === "number") {
        return typeof value === "number";
    } else if (type === "boolean") {
        return typeof value === "boolean";
    }
}

function EnumDefinition(states: object) {
    this.states = states;
    this.ids = {};
}

function assert(didPass: boolean, type: any, value: any) {
    if (!didPass) {
        throw new Error("Enum state created with invalid values! " + JSON.stringify(value) + " does not follow type of " + JSON.stringify(type));
    }
}

export function createEnumDefinition(states: {
    [key: string]: any
}) {
    states = Object.freeze(states);
    var newEnumDef = new (EnumDefinition as any)(states);
    var stateUid = 0;

    // we are creating functions for each of the enum's states
    // these functions return an EnumInstance
    // these functions will verify that when you create an
    // enum, it'll have the correct types passed in

    for (let state in states) {
        // state === the state id currently having a function
        // built for it, the function for creating an enum instance
        // of type state
        newEnumDef.ids[state] = stateUid++;

        // create new scope (IIFE)
        (function () {
            var currentStateId = state; // state is 'string', so this is a copy
            var currentState = states[currentStateId];
            var enumId = newEnumDef.ids[currentStateId];

            // this is an empty state, so you shouldn't have to call
            // a function to create it, just use Enum.StateWithoutValue
            if (currentState === null) {
                newEnumDef[currentStateId] = new EnumInstance(newEnumDef, enumId, null);
            } else {
                // otherwise it's a function, so you can use Enum.State(foo, bar)

                if (typeof currentState === "string") {
                    currentState = [currentState];
                }

                if (Array.isArray(currentState)) {
                    newEnumDef[currentStateId] = function (args: any) {
                        if (arguments.length > 1 || !Array.isArray(args)) {
                            args = Array.from(arguments);
                        }

                        // verify all the incoming values
                        assert(args.length === currentState.length, currentState, args);

                        for (var i = 0; i < currentState.length; i++) {
                            assert(verifyInput(currentState[i], args[i]), currentState[i], args[i]);
                        }

                        return new EnumInstance(newEnumDef, enumId, args);
                    };
                } else if (typeof currentState === "object") {
                    newEnumDef[currentStateId] = function (args: any) {
                        // verify all the incoming values
                        for (var prop in currentState) {
                            assert(verifyInput(currentState[prop], args[prop]), currentState[prop], args[prop]);
                        }

                        return new EnumInstance(newEnumDef, enumId, args);
                    }
                }
            }
        })();
    }

    // catch all match
    newEnumDef.ids["_"] = -1;

    return newEnumDef;
}
