import type { Socket } from "./connection";
import { match } from "../util/discriminated-union";


export class SocketRegistry {
    nameToSocketType: {[key: string]: number};

    constructor () {
        this.nameToSocketType = {};
    }

    applyJson (json: any) {
        this.nameToSocketType = {
            ...this.nameToSocketType,
            ...json.nameToSocketType
        };
    }

    getSocketInterpolation (socket: Socket): [string, any] {
        return match(socket, {
            Simple: ({ data: [uidLookingFor] }) => {
                const socketFound = Object.entries(this.nameToSocketType).find(([_, uid]) => uid === uidLookingFor);

                if (socketFound) {
                    return [socketFound[0], undefined];
                }

                return ["", undefined];
            },
            Numbered: ({ data: [uidLookingFor, associatedData] }) => {
                const socketFound = Object.entries(this.nameToSocketType).find(([_, uid]) => uid === uidLookingFor);

                if (socketFound) {
                    return [socketFound[0], associatedData];
                }

                return ["", undefined];
            }
        });
    }
}
