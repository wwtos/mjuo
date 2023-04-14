<script lang="ts">
    import { onMount } from "svelte";
    import type { Writable } from "svelte/store";

    export let inputMidiDevice: MIDIInput | undefined = undefined;
    export let outputMidiDevice: MIDIOutput | undefined = undefined;

    export let listInputs = false;
    export let listOutputs = false;

    let midi: MIDIAccess;

    let midiInputs: Array<[string, MIDIInput]> = [];
    let midiOutputs: Array<[string, MIDIOutput]> = [];

    let inputDeviceId: string;
    let outputDeviceId: string;

    function initDeviceList() {
        if (listInputs) {
            midiInputs = [...midi.inputs.entries()];
        }

        if (listOutputs) {
            midiOutputs = [...midi.outputs.entries()];
        }
    }

    function onMIDISuccess(midiAccess: MIDIAccess) {
        midi = midiAccess;

        midi.addEventListener("statechange", initDeviceList);

        initDeviceList();
    }

    function onMIDIFailure() {
        alert("Cannot run this program without MIDI access.");
    }

    function midiInputChanged() {
        if (listInputs) {
            const device = midi.inputs.get(inputDeviceId);
            if (device) inputMidiDevice = device;
        }
    }

    function midiOutputChanged() {
        if (listOutputs) {
            const device = midi.outputs.get(outputDeviceId);
            if (device) outputMidiDevice = device;
        }
    }

    onMount(async () => {
        navigator.requestMIDIAccess().then(onMIDISuccess, onMIDIFailure);
    });
</script>

{#if listInputs}
    Inputs:
    <select bind:value={inputDeviceId} on:change={midiInputChanged}>
        {#each midiInputs as device}
            <option value={device[0]}>{device[1].name}</option>
        {/each}
    </select>
{/if}

{#if listOutputs}
    Outputs:
    <select bind:value={outputDeviceId} on:change={midiOutputChanged}>
        {#each midiOutputs as device}
            <option value={device[0]}>{device[1].name}</option>
        {/each}
    </select>
{/if}
