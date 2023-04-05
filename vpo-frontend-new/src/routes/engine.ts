import workletUrl from "$lib/engine/engine_worklet.ts?url";
import wasmUrl from "$lib/wasm/vpo_backend_bg.wasm?url";

export class Engine {
    context: AudioContext;
    worklet: AudioWorkletNode;

    constructor(context: AudioContext, worklet: AudioWorkletNode) {
        this.context = context;
        this.worklet = worklet;
    }

    send(message: string) {
        this.worklet.port.postMessage({
            type: "message",
            payload: message
        });
    }
}

export async function constructEngine(context: AudioContext) {
    await context.audioWorklet.addModule(workletUrl);
    const module = await fetch(wasmUrl).then(res => res.arrayBuffer());

    const worklet = new AudioWorkletNode(context, "RustEngineWorklet", {
        processorOptions: { module },
    });

    worklet.connect(context.destination);

    return new Engine(context, worklet);
}