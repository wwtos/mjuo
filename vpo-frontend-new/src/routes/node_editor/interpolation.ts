import { BehaviorSubject, Observable } from "rxjs";
import type { SocketType } from "$lib/node-engine/connection";
import { match } from "$lib/util/discriminated-union";
import type { SocketRegistry } from "$lib/node-engine/socket_registry";

export function socketTypeToString(registry: SocketRegistry, socketType: SocketType, localize: Function): BehaviorSubject<string> {
    let response = match(socketType, {
        Stream: ({ data: stream }): string | Observable<string> => match(stream, {
            Audio: () => localize("socket-audio"),
            Gate: () => localize("socketType.stream.gate"),
            Gain: () => localize("socketType.stream.gain"),
            Detune: () => localize("socketType.stream.detune"),
            Dynamic: ({ data: uid }) => registry.getSocketInterpolation(uid)
        }),
        Midi: ({ data: midi }): string | Observable<string> => match(midi, {
            Default: () => localize("socketType.midi.default"),
            Dynamic: ({ data: uid }) => registry.getSocketInterpolation(uid)
        }),
        Value: ({ data: value }): string | Observable<string> => match(value, {
            Default: () => localize("socketType.value.default"),
            Gain: () => localize("socketType.value.gain"),
            Frequency: () => localize("socketType.value.frequency"),
            Resonance: () => localize("socketType.value.resonance"),
            Gate: () => localize("socketType.value.gate"),
            Attack: () => localize("socketType.value.attack"),
            Decay: () => localize("socketType.value.decay"),
            Sustain: () => localize("socketType.value.sustain"),
            Release: () => localize("socketType.value.release"),
            Speed: () => localize("socketType.value.speed"),
            State: () => localize("socketType.value.state"),
            UiState: () => localize("socketType.value.uiState"),
            Dynamic: ({ data: uid }) => registry.getSocketInterpolation(uid)
        }),
        NodeRef: ({ data: nodeRef }): string | Observable<string> => match(nodeRef, {
            Button: () => localize("socketType.noderef.button"),
            Dynamic: ({ data: uid }) => registry.getSocketInterpolation(uid)
        }),
        MethodCall: () => "Method call",
    });

    if (response instanceof Observable) {
        let bh = new BehaviorSubject<string>("");
        response.subscribe(bh);

        return bh;
    } else {
        return new BehaviorSubject<string>(response);
    }
}
