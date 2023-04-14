<script lang="ts">
    import DeviceSelector from "$lib/components/DeviceSelector.svelte";
    import type { WasmIpcSocket } from "$lib/ipc/socket";

    export let socket: WasmIpcSocket;

    let inputMidiDevice: MIDIInput | undefined;
    let lastMidiDevice: MIDIInput | undefined;

    $: if (inputMidiDevice) {
        if (lastMidiDevice) lastMidiDevice.onmidimessage = () => {};

        inputMidiDevice.onmidimessage = (event) => {
            const data = (event as MIDIMessageEvent).data;

            socket.sendRaw({
                type: "midi",
                payload: data,
            });
        };

        lastMidiDevice = inputMidiDevice;
    }
</script>

<div>
    Select a device: <DeviceSelector
        listInputs={true}
        listOutputs={false}
        bind:inputMidiDevice
    />
</div>
