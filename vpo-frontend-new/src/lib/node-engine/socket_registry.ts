import { BehaviorSubject, Observable } from "rxjs";
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
            ...this.nameToSocketType$.getValue(),
            ...json.name_to_socket_type
        };

        this.nameToSocketType$.next(newNameToSocketType);
    }

    getRegistryValue (name: string): Observable<RegistryValue | undefined> {
        return this.nameToSocketType$.pipe(
            map(nameToSocketType => nameToSocketType[name])
        );
    }

    getSocketInterpolation (uidToLookFor: number): Observable<[string, any]> {
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
                    return ["socket-" + entry.template, entry.associatedData];
                } else {
                    return ["", undefined];
                }
            })
        );
    }
}
