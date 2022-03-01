<script lang="ts">
    import Socket from "./Socket.svelte";
    import { onMount } from 'svelte';
    import {storeWatcher} from "../util/store-watcher";
    import { NodeIndex, NodeWrapper } from "../node-engine/node";
    import { EnumInstance } from "../util/enum";
    import { SocketType, StreamSocketType } from "../node-engine/connection";

    const ROUNDNESS = 7;
    const PADDING = 10;
    const PADDING_TOP = PADDING + 7;
    const TEXT_PADDING = 30;
    const SOCKET_LIST_START = 55;
    const TEXT_SIZE = 14;
    const SOCKET_VERTICAL_SPACING = TEXT_SIZE + 5;
    
    export let title = "Test title";
    export let properties = [
        ["Audio in", 
        {
            "type": "Stream",
            "content": [{
                "type": "Audio"
            }]
        }, "INPUT"],
        ["Audio out", 
        {
            "type": "Value",
            "content": [{
                "type": "Audio"
            }]
        }, "OUTPUT"]
    ];

    let wrapper: NodeWrapper = {
        node: {
            inputSockets: [StreamSocketType.Audio],
            outputSockets: [StreamSocketType.Audio],
            listProperties: function(): EnumInstance[] {
                return [];
            },
            serializeToJson: function(): object {
                return {}
            },
            applyJson: function(json) {}
        },
        index: new NodeIndex(0, 0),
        connectedInputs: [],
        connectedOutputs: []
    };

    export let width = 200;
    export let x = 100;
    export let y = 100;

    export let onMousedown = function() {};

    let computedHeight = SOCKET_LIST_START + SOCKET_VERTICAL_SPACING * (properties.length - 1) + TEXT_SIZE + PADDING;

    let dragging = false;
    let dragAnchor = {x: 0, y: 0};
</script>

<g transform="translate({x}, {y})">
<rect width="{width}" height="{computedHeight}" rx="{ROUNDNESS}" class="background" on:mousedown={onMousedown} />
<text x={PADDING} y={PADDING_TOP} class="title" on:mousedown={onMousedown}>{title}</text>

{#each [...wrapper.node.inputSockets, ...wrapper.node.outputSockets] as socket, i (index.toString())}
    {#if i < wrapper.node.inputSockets.length}
        <text x={TEXT_PADDING} y={SOCKET_LIST_START + SOCKET_VERTICAL_SPACING * i} on:mousedown={onMousedown}>{property[0]}</text>
        <Socket x={0} y={SOCKET_LIST_START + SOCKET_VERTICAL_SPACING * i} type={property[1].type} />
    {:else}
        <text x={width - TEXT_PADDING} y={SOCKET_LIST_START + SOCKET_VERTICAL_SPACING * i} class="right-align" on:mousedown={onMousedown}>{property[0]}</text>
        <Socket x={width} y={SOCKET_LIST_START + SOCKET_VERTICAL_SPACING * i} type={property[1].type} />
    {/if}
{/each}
</g>

<style>
.background {
    fill: rgba(110, 136, 255, 0.8);
    stroke: #4e58bf;
    stroke-width: 2px;
}

text {
    text-anchor: start;
    dominant-baseline: central;
    font-size: 14px;
    font-family: sans-serif;
    fill: white;
    user-select: none;
}

.right-align {
    text-anchor: end;
}

.title {
    font-size: 18px;
}
</style>