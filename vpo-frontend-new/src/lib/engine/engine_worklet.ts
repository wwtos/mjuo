import { initSync, State } from "../wasm/vpo_backend";

class RustEngineWorklet extends AudioWorkletProcessor {
    state: State;
    toInput: string | undefined;
    midiIn: Uint8Array;

    constructor(options?: AudioWorkletNodeOptions) {
        super();

        // init the wasm module
        let { module } = options?.processorOptions;
        initSync(module);

        this.state = State.new(44100);
        this.midiIn = new Uint8Array();

        this.port.onmessage = (event) => {
            let type = event.data.type;

            switch (type) {
                case "midi":
                    throw new Error("unimplemented");
                    break;
                case "message":
                    this.toInput = event.data.payload;
                    break;
            }
        };
    }

    process(_inputs: Float32Array[][], outputs: Float32Array[][]) {
        let result = this.state.step(this.toInput, this.midiIn, outputs[0][0]);

        this.toInput = undefined;
        this.midiIn = new Uint8Array();

        for (let i = 1; i < outputs[0].length; i++) {
            for (let j = 0; j < outputs[0][i].length; j++) {
                outputs[0][i][j] = outputs[0][0][j];
            }
        }

        if (result.length > 0) {
            this.port.postMessage(result);
        }

        return true;
    }
}

registerProcessor("RustEngineWorklet", RustEngineWorklet);

// to make typescript happy
export type {}
