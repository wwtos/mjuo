<script lang="ts">
    import type { IpcSocket } from "$lib/ipc/socket";
    import type {
        AudioDeviceStatus,
        GlobalState,
        MidiDeviceStatus,
    } from "$lib/node-engine/global_state";
    import type { DeviceInfo, RouteRule } from "$lib/node-engine/io_routing";
    import type { Writable } from "svelte/store";
    import type { VertexIndex } from "$lib/ddgg/graph";
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import type { BehaviorSubject } from "rxjs";

    export let socket: IpcSocket;
    export let globalState: Writable<GlobalState>;
    export let activeGraph: BehaviorSubject<NodeGraph>;

    $: streams = $globalState.devices.streams;
    $: midi = $globalState.devices.midi;
    $: routes = $globalState.ioRoutes;
    $: graph = $activeGraph;

    let midiDeviceName: string;
    let midiDeviceDirection: "Source" | "Sink" = "Source";

    let audioDeviceName: string;
    let audioDeviceDirection: "Source" | "Sink" = "Sink";
    let bufferSize: number = 1024;
    let minBufferSize: number;
    let maxBufferSize: number;
    let channels: number = 2;
    let minChannels: number;
    let maxChannels: number;

    let midiDevice: MidiDeviceStatus | undefined;
    $: midiDevice = midi[midiDeviceName];

    let audioDevice: AudioDeviceStatus | undefined;
    $: audioDevice = streams[audioDeviceName];

    let routeRuleDeviceAsStr: string = "{}";
    let routeRuleDevice: DeviceInfo;
    let routeRuleDeviceChannel: number = 0;
    let routeRuleNode: VertexIndex;
    let routeRuleNodeChannel: number = 0;
    $: routeRuleDevice = JSON.parse(routeRuleDeviceAsStr);

    $: if (audioDevice) {
        if (audioDeviceDirection === "Source") {
            minBufferSize = audioDevice.sourceOptions?.buffer_size.start || 0;
            maxBufferSize = audioDevice.sourceOptions?.buffer_size.end || 0;
            minChannels = audioDevice.sourceOptions?.channels.start || 0;
            maxChannels = audioDevice.sourceOptions?.channels.end || 0;
        } else {
            minBufferSize = audioDevice.sinkOptions?.buffer_size.start || 0;
            maxBufferSize = audioDevice.sinkOptions?.buffer_size.end || 0;
            minChannels = audioDevice.sinkOptions?.channels.start || 0;
            maxChannels = audioDevice.sinkOptions?.channels.end || 0;
        }

        if (bufferSize < minBufferSize) {
            bufferSize = minBufferSize;
        } else if (bufferSize > maxBufferSize) {
            bufferSize = maxBufferSize;
        }
    }

    function addAudioDevice() {
        let newDevice: DeviceInfo = {
            name: audioDeviceName,
            deviceType: { variant: "Stream" },
            deviceDirection: {
                variant: audioDeviceDirection,
            },
            channels: channels,
            bufferSize: bufferSize,
        };

        socket.commit([
            {
                variant: "ChangeRouteRules",
                data: {
                    newRules: {
                        devices: [...routes.devices, newDevice],
                        rules: [...routes.rules],
                    },
                },
            },
        ]);
    }

    function addMidiDevice() {
        let newDevice: DeviceInfo = {
            name: midiDeviceName,
            deviceType: { variant: "Midi" },
            deviceDirection: { variant: midiDeviceDirection },
            channels: 0,
            bufferSize: 0,
        };

        socket.commit([
            {
                variant: "ChangeRouteRules",
                data: {
                    newRules: {
                        devices: [...routes.devices, newDevice],
                        rules: [...routes.rules],
                    },
                },
            },
        ]);
    }

    function addRouteRule() {
        let newRouteRule: RouteRule = {
            deviceId: routeRuleDevice.name,
            deviceDirection: routeRuleDevice.deviceDirection,
            deviceType: routeRuleDevice.deviceType,
            deviceChannel: routeRuleDeviceChannel,
            node: routeRuleNode,
            nodeChannel: routeRuleNodeChannel,
        };

        socket.commit([
            {
                variant: "ChangeRouteRules",
                data: {
                    newRules: {
                        devices: [...routes.devices],
                        rules: [...routes.rules, newRouteRule],
                    },
                },
            },
        ]);
    }

    function disconnectDevice(deviceToRemove: DeviceInfo) {
        let newDevices = routes.devices.filter(
            (device) => device !== deviceToRemove,
        );

        socket.commit([
            {
                variant: "ChangeRouteRules",
                data: {
                    newRules: {
                        devices: newDevices,
                        rules: routes.rules,
                    },
                },
            },
        ]);
    }

    function removeRouteRule(ruleToRemove: RouteRule) {
        let newRules = routes.rules.filter((rule) => rule !== ruleToRemove);

        socket.commit([
            {
                variant: "ChangeRouteRules",
                data: {
                    newRules: {
                        devices: routes.devices,
                        rules: newRules,
                    },
                },
            },
        ]);
    }
</script>

<div>
    <p>
        <strong>Add audio device:</strong>
        <select bind:value={audioDeviceName}>
            <option></option>
            {#each Object.keys(streams) as device}
                <option value={device}>{device}</option>
            {/each}
        </select>
        {#if audioDevice}
            <select bind:value={audioDeviceDirection}>
                {#if audioDevice.sourceOptions !== null}
                    <option value="Source">Input</option>
                {/if}
                {#if audioDevice.sinkOptions !== null}
                    <option value="Sink">Output</option>
                {/if}
            </select>
            Buffer size:
            <input
                type="number"
                bind:value={bufferSize}
                min={minBufferSize}
                max={maxBufferSize}
            />
            Channels ({minChannels}-{maxChannels}):
            <input
                type="number"
                bind:value={channels}
                min={minChannels}
                max={maxChannels}
            />
            <button on:click={addAudioDevice}> create </button>
        {/if}
    </p>
    <p>
        <strong>Add midi device:</strong>
        <select bind:value={midiDeviceName}>
            <option></option>
            {#each Object.keys(midi) as device}
                <option>{device}</option>
            {/each}
        </select>
        {#if midiDevice}
            <select bind:value={midiDeviceDirection}>
                <option value="Source">Input</option>
                <option value="Sink">Output</option>
            </select>
            <button on:click={addMidiDevice}> create </button>
        {/if}
    </p>
    <fieldset>
        <legend>Add route rule:</legend> device:
        <select bind:value={routeRuleDeviceAsStr}>
            <option value={"{}"}></option>
            {#each routes.devices as device}
                <option value={JSON.stringify(device)}>
                    "{device.name}"
                    {device.deviceType.variant}
                    {device.deviceDirection.variant}
                </option>
            {/each}
        </select>
        {#if routeRuleDevice.deviceType}
            <br />
            node:

            <select bind:value={routeRuleNode}>
                <option></option>
                {#each graph.getNodes() as [nodeId, node] (nodeId)}
                    {#if routeRuleDevice.deviceDirection.variant === "Sink" && node.nodeType === "OutputsNode"}
                        <option value={nodeId}>
                            {node.properties["name"].data}
                        </option>
                    {/if}
                    <!-- can't get formatter to work right, so I'm making this `if` twice -->
                    {#if routeRuleDevice.deviceDirection.variant === "Source" && node.nodeType === "InputsNode"}
                        <option value={nodeId}>
                            {node.properties["name"].data}
                        </option>
                    {/if}
                {/each}
            </select>
            <br />
            {#if routeRuleDevice?.deviceType?.variant === "Stream"}
                device channel:
                <input type="number" bind:value={routeRuleDeviceChannel} />
                <br />

                node channel:
                <input type="number" bind:value={routeRuleNodeChannel} />
                <br />
            {/if}
            <button on:click={addRouteRule}>create</button>
        {/if}
    </fieldset>
    <br />
    <div>
        Connected devices:
        <ul>
            {#each routes.devices as device}
                {#if device.deviceType.variant === "Midi"}
                    <li>
                        Midi: "{device.name}", {device.deviceDirection.variant}
                        <button on:click={() => disconnectDevice(device)}>
                            disconnect
                        </button>
                    </li>
                {:else}
                    <li>
                        Stream: "{device.name}", {device.deviceDirection
                            .variant}, buffer size: {device.bufferSize},
                        channels: {device.channels}
                        <button on:click={() => disconnectDevice(device)}>
                            disconnect
                        </button>
                    </li>
                {/if}
            {/each}
        </ul>
    </div>
    <div>
        Route rules:
        <ul>
            {#each routes.rules as rule (rule)}
                <li>
                    "{rule.deviceId}",
                    {rule.deviceType.variant},
                    {rule.deviceDirection.variant}, device channel {rule.deviceChannel}
                    &lt;-&gt; node channel {rule.nodeChannel}, node "{graph.getNode(
                        rule.node,
                    )?.properties["name"].data}"
                    <button on:click={() => removeRouteRule(rule)}>
                        delete
                    </button>
                </li>
            {/each}
        </ul>
    </div>
</div>
<div></div>

<style>
    div {
        display: block;
    }
</style>
