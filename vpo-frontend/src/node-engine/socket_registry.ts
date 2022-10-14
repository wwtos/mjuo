import { BehaviorSubject, Observable } from "rxjs";
import { i18n } from "../i18n";
import type { SocketType } from "./connection";
import { map } from "rxjs/operators";
import { matchOrElse } from "../util/discriminated-union";

interface RegistryValue {
    template: string;
    socketType: SocketType;
    associatedData: any;
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
            newNameToSocketType[key] = json.name_to_socket_type[key];
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
                    return matchOrElse(entry.socketType, {
                        Stream: ({ data: stream }) => matchOrElse(stream, {
                            Dynamic: ({ data: uid }) => uidToLookFor === uid,
                        },  () => false),
                        Midi: ({data: midi }) => matchOrElse(midi, {
                            Dynamic: ({ data: uid }) => uidToLookFor === uid,
                        },  () => false),
                        Value: ({ data: value }) => matchOrElse(value, {
                            Dynamic: ({ data: uid }) => uidToLookFor === uid,
                        },  () => false),
                        NodeRef: ({ data: nodeRef }) => matchOrElse(nodeRef, {
                            Dynamic: ({ data: uid }) => uidToLookFor === uid,
                        },  () => false),
                    }, () => false);
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
