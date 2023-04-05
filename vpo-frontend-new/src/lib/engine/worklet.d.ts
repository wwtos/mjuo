// provide missing typescript types for audio worklets
// https://github.com/microsoft/TypeScript/issues/28308#issuecomment-650802278
interface AudioWorkletProcessor {
    readonly port: MessagePort;
    process(
      inputs: Float32Array[][],
      outputs: Float32Array[][],
      parameters: Record<string, Float32Array>
    ): boolean;
}
  
declare var AudioWorkletProcessor: {
    prototype: AudioWorkletProcessor;
    new (options?: AudioWorkletNodeOptions): AudioWorkletProcessor;
};

declare var currentTime: number;
  
declare function registerProcessor(
    name: string,
    processorCtor: (new (
      options?: AudioWorkletNodeOptions
    ) => AudioWorkletProcessor) & {
      parameterDescriptors?: AudioParamDescriptor[];
    }
): void;