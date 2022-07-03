import { MemberType } from "safety-match";
import { jsonToSocketType, SocketType } from "./connection";

class RegistryValue {
    template: string;
    socketType: MemberType<typeof SocketType>;
    associatedData: any;

    constructor (json: any) {
        this.template = json.template;
        this.socketType = jsonToSocketType(json.socket_type);
        this.associatedData = json.associated_data;
    }
}

export class SocketRegistry {
    nameToSocketType: {[key: string]: RegistryValue};

    constructor () {
        this.nameToSocketType = {};
    }

    applyJson (json: any) {
        for (let key in json.name_to_socket_type) {
            this.nameToSocketType[key] = new RegistryValue(json.name_to_socket_type[key]);
        }
    }
}