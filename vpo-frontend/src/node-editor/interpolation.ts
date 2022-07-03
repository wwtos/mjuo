import { BehaviorSubject, Observable } from "rxjs";
import { MemberType } from "safety-match";
import { i18n } from "../i18n";
import { SocketType } from "../node-engine/connection";
import { socketRegistry } from "./state";

export function socketTypeToString(socketType: MemberType<typeof SocketType>): BehaviorSubject<string> {
    let response = socketType.match({
        Stream: (stream) => stream.match({
            Audio: () => i18n.t("socketType.stream.audio"),
            Gate: () => i18n.t("socketType.stream.gate"),
            Gain: () => i18n.t("socketType.stream.gain"),
            Detune: () => i18n.t("socketType.stream.detune"),
            Dynamic: (uid) => socketRegistry.getValue().getSocketInterpolation(uid)
        }),
        Midi: (midi) => midi.match({
            Default: () => i18n.t("socketType.midi.default"),
            Dynamic: (uid) => i18n.t("socketType.midi.dynamic", { uid })
        }),
        Value: (value) => value.match({
            Gain: () => i18n.t("socketType.value.gain"),
            Frequency: () => i18n.t("socketType.value.frequency"),
            Resonance: () => i18n.t("socketType.value.resonance"),
            Gate: () => i18n.t("socketType.value.gate"),
            Attack: () => i18n.t("socketType.value.attack"),
            Decay: () => i18n.t("socketType.value.decay"),
            Sustain: () => i18n.t("socketType.value.sustain"),
            Release: () => i18n.t("socketType.value.release"),
            Dynamic: (uid) => i18n.t("socketType.value.dynamic", { uid })
        }),
        NodeRef: (nodeRef) => nodeRef.match({
            Button: () => i18n.t("socketType.noderef.button"),
            Dynamic: (uid) => i18n.t("socketType.noderef.dynamic", { uid })
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