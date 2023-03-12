import { State } from "../../vpo-backend/pkg/vpo_backend";

class WasmEngineProcessor extends AudioWorkletProcessor {
    engine: State;

    constructor(context) {
        super();

        this.port.onmessage = (event) => {
            console.log(event);
        };

        this.engine = State.new(context.sampleRate);
    }

    process(_inputs, outputs, parameters) {
        //this.engine.step()
        this.port.postMessage("response", "hello world");        

        return true;
    }
}

registerProcessor("wasm-engine-processor", WasmEngineProcessor);