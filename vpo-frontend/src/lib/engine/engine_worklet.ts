import { initSync, State } from "../wasm/vpo_backend";

class RustEngineWorklet extends AudioWorkletProcessor {
    state: State;
    toInput: string[];
    midiIn: Uint8Array[];

    constructor(options?: AudioWorkletNodeOptions) {
        super();

        // init the wasm module
        let { module } = options?.processorOptions;
        initSync(module);

        this.state = State.new(48000);
        this.midiIn = [];

        this.port.onmessage = (event) => {
            const data = event.data.payload;
            let type = data.type;

            switch (type) {
                case "midi":
                    this.midiIn.push(data.payload);
                    break;
                case "resource":
                    const resource = new Uint8Array(data.resource);
                    const associatedResource = data.associatedResource && new Uint8Array(data.associatedResource);

                    let err = this.state.load_resource(data.path, resource, associatedResource);

                    if (err) {
                        console.log("loading error: ", err);
                    }
                    break;
                case "message":
                    this.toInput.push(JSON.stringify(data.payload));
                    break;
            }
        };

        this.toInput = [];
    }

    process(_inputs: Float32Array[][], outputs: Float32Array[][]) {
        let result = this.state.step(this.toInput.splice(0, 1)[0], this.midiIn.splice(0, 1)[0] ?? new Uint8Array(), outputs[0][0]);

        if (result.length > 0) {
            this.port.postMessage(result);
        }

        for (let i = 1; i < outputs[0].length; i++) {
            for (let j = 0; j < outputs[0][i].length; j++) {
                outputs[0][i][j] = outputs[0][0][j];
            }
        }

        return true;
    }
}

registerProcessor("RustEngineWorklet", RustEngineWorklet);

// to make typescript happy
export type {}
