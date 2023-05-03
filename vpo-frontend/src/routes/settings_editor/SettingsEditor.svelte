<script lang="ts">
    import DeviceSelector from "$lib/components/DeviceSelector.svelte";
    import type { IpcSocket, WasmIpcSocket } from "$lib/ipc/socket";

    export let socket: IpcSocket;

    let inputMidiDevice: MIDIInput | undefined;
    let lastMidiDevice: MIDIInput | undefined;

    let socketIsWasm = (socket as any).sendRaw;

    $: if (inputMidiDevice && socketIsWasm) {
        if (lastMidiDevice) lastMidiDevice.onmidimessage = () => {};

        inputMidiDevice.onmidimessage = (event) => {
            const data = (event as MIDIMessageEvent).data;

            (socket as WasmIpcSocket).sendRaw({
                type: "midi",
                payload: data,
            });
        };

        lastMidiDevice = inputMidiDevice;
    }
</script>

<div>
    {#if socketIsWasm}
        Select a device: <DeviceSelector
            listInputs={true}
            listOutputs={false}
            bind:inputMidiDevice
        />
    {/if}
</div>
