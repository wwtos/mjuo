<script lang="ts">
    import Socket from "./Socket.svelte";
    import { onMount } from 'svelte';
    import {storeWatcher} from "../util/store-watcher";
    import { NodeIndex, NodeWrapper } from "../node-engine/node";
    import { EnumInstance } from "../util/enum";
    import { SocketType, StreamSocketType, MidiSocketType, SocketDirection, socketTypeToString } from "../node-engine/connection";
    
    export let title = "Test title";

    export let wrapper: NodeWrapper/* = {
        node: {
            inputSockets: [SocketType.Midi(MidiSocketType.Default)],
            outputSockets: [SocketType.Stream(StreamSocketType.Audio)],
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
    }*/;

    console.log("Wrapper", wrapper);

    let sockets: any[][];

    $: sockets = [
        ...wrapper.node.inputSockets.map(inputSocket => [inputSocket, SocketDirection.Input]),
        ...wrapper.node.outputSockets.map(outputSocket => [outputSocket, SocketDirection.Output]),
    ];

    export let width = 200;
    
    export let onMousedown = function(index: NodeIndex, e: MouseEvent) {};

    function onMousedownRaw (e: MouseEvent) {
        onMousedown(new NodeIndex(wrapper.index.index, wrapper.index.generation), e);
    }

    let x, y, selected;

    $: x = wrapper.uiData.x;
    $: y = wrapper.uiData.y;
    $: selected = wrapper.uiData.selected;
</script>

<div class="background" style="transform: translate({x}px, {y}px); width: {width}px" on:mousedown={onMousedownRaw} class:selected={selected}>
    <div class="node-title">{title}</div>

    {#each sockets as [socket, direction] (socket.getType() + "" + direction)}
        <Socket type={socket.getType()} direction={direction} label={socketTypeToString(socket)} />
    {/each}
</div>

<style>
.node-title {
    color: white;
    font-size: 18px;
    margin: 8px;
}
.background {
    position: absolute;
    background-color: rgba(110, 136, 255, 0.8);
    border: solid 2px #4e58bf;
    border-radius: 7px;
    text-align: left;
    font-size: 14px;
    font-family: sans-serif;
    fill: white;
    user-select: none;
    z-index: -10;
}

.background.selected {
    background-color: rgba(110, 226, 255, 0.8);
}

.right-align {
    text-anchor: end;
}

.title {
    font-size: 18px;
}
</style>