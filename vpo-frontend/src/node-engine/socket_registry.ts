import { BehaviorSubject, Observable } from "rxjs";
import { MemberType } from "safety-match";
import { i18n, i18n$ } from "../i18n";
import { jsonToSocketType, SocketType } from "./connection";
import { map } from "rxjs/operators";

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
    nameToSocketType$: BehaviorSubject<{[key: string]: RegistryValue}>;

    constructor () {
        this.nameToSocketType$ = new BehaviorSubject({});
    }

    applyJson (json: any) {
        let newNameToSocketType = {
            ...this.nameToSocketType$.getValue()
        }

        for (let key in json.name_to_socket_type) {
            newNameToSocketType[key] = new RegistryValue(json.name_to_socket_type[key]);
        }

        this.nameToSocketType$.next(newNameToSocketType);
    }

    getRegistryValue (name: string): Observable<RegistryValue | undefined> {
        return this.nameToSocketType$.pipe(
            map(nameToSocketType => nameToSocketType[name])
        );
    }

    getSocketInterpolation (uidToLookFor: number): Observable<string> {
        return this.nameToSocketType$.pipe(
            map(nameToSocketType => {
                const entry = Object.values(nameToSocketType).find(entry => {
                    return entry.socketType.match({
                        Stream: (stream) => stream.match({
                            Dynamic: (uid) => uidToLookFor === uid,
                            _: () => false
                        }),
                        Midi: (midi) => {
                            return midi.match({
                            Dynamic: (uid) => uidToLookFor === uid,
                            _: () => false
                        })},
                        Value: (stream) => stream.match({
                            Dynamic: (uid) => uidToLookFor === uid,
                            _: () => false
                        }),
                        NodeRef: (stream) => stream.match({
                            Dynamic: (uid) => uidToLookFor === uid,
                            _: () => false
                        }),
                        _: () => false
                    });
                });

                if (entry) {
                    return i18n.t("customSockets." + entry.template, entry.associatedData);
                } else {
                    return "";
                }
            })
        );
    }
}
