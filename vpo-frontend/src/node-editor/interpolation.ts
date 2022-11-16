import { BehaviorSubject, Observable } from "rxjs";
import { MemberType } from "safety-match";
import { i18n } from "../i18n";
import { SocketType } from "../node-engine/connection";
import { match } from "../util/discriminated-union";
import { socketRegistry } from "./state";

export function socketTypeToString(socketType: SocketType): BehaviorSubject<string> {
    let response = match(socketType, {
        Stream: ({ data: stream }): string | Observable<string> => match(stream, {
            Audio: () => i18n.t("socketType.stream.audio"),
            Gate: () => i18n.t("socketType.stream.gate"),
            Gain: () => i18n.t("socketType.stream.gain"),
            Detune: () => i18n.t("socketType.stream.detune"),
            Dynamic: ({ data: uid }) => socketRegistry.getValue().getSocketInterpolation(uid)
        }),
        Midi: ({ data: midi }): string | Observable<string> => match(midi, {
            Default: () => i18n.t("socketType.midi.default"),
            Dynamic: ({ data: uid }) => socketRegistry.getValue().getSocketInterpolation(uid)
        }),
        Value: ({ data: value }): string | Observable<string> => match(value, {
            Default: () => i18n.t("socketType.value.default"),
            Gain: () => i18n.t("socketType.value.gain"),
            Frequency: () => i18n.t("socketType.value.frequency"),
            Resonance: () => i18n.t("socketType.value.resonance"),
            Gate: () => i18n.t("socketType.value.gate"),
            Attack: () => i18n.t("socketType.value.attack"),
            Decay: () => i18n.t("socketType.value.decay"),
            Sustain: () => i18n.t("socketType.value.sustain"),
            Release: () => i18n.t("socketType.value.release"),
            Speed: () => i18n.t("socketType.value.speed"),
            Dynamic: ({ data: uid }) => socketRegistry.getValue().getSocketInterpolation(uid)
        }),
        NodeRef: ({ data: nodeRef }): string | Observable<string> => match(nodeRef, {
            Button: () => i18n.t("socketType.noderef.button"),
            Dynamic: ({ data: uid }) => socketRegistry.getValue().getSocketInterpolation(uid)
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
