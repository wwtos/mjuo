<script lang="ts">
    import NodeRowUI from "./NodeRow.svelte";
    import { onMount } from 'svelte';
    import {storeWatcher} from "../util/store-watcher";
    import { NodeIndex, NodeRow, NodeWrapper } from "../node-engine/node";
    import { EnumInstance } from "../util/enum";
    import { SocketType, StreamSocketType, MidiSocketType, SocketDirection, socketTypeToString, socketTypeToKey, socketToKey } from "../node-engine/connection";

    // in pixels, these numbers are derived from the css below and the css in ./Socket.svelte
    const TITLE_HEIGHT = 30;
    const SOCKET_HEIGHT = 36;
    const SOCKET_OFFSET = -10;

    export let width = 200;

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

    export let exportSocketPositionMapping = function(
        socketPositionMapping: ({ [key: string]: [number, number] }),
        key: string
    ) {};

    let sockets: any[][];

    $: sockets = wrapper.nodeRows.map(nodeRow => {
        return nodeRow.match([
            [NodeRow.ids.StreamInput, ([streamInput, def]) => [SocketType.Stream(streamInput), SocketDirection.Input, def]],
            [NodeRow.ids.MidiInput, ([midiInput, def]) => [SocketType.Midi(midiInput), SocketDirection.Input, def]],
            [NodeRow.ids.ValueInput, ([valueInput, def]) => [SocketType.Value(valueInput), SocketDirection.Input, def]],
            [NodeRow.ids.StreamOutput, ([streamOutput, def]) => [SocketType.Stream(streamOutput), SocketDirection.Output, def]],
            [NodeRow.ids.MidiOutput, ([midiOutput, def]) => [SocketType.Midi(midiOutput), SocketDirection.Output, def]],
            [NodeRow.ids.ValueOutput, ([valueOutput, def]) => [SocketType.Value(valueOutput), SocketDirection.Output, def]],
            [NodeRow.ids._, () => {}]
        ]);
    }).filter(maybeSomething => !!maybeSomething);

    let node: HTMLDivElement;

    // whenever the socket list changes, update the relative positions of all the sockets
    $: {
        let socketPositionMapping: ({
            [key: string]: [number, number]
        }) = {};

        let y = TITLE_HEIGHT;

        for (let socket of sockets) {
            const socketKey = socketToKey(socket[0], socket[1]);

            y += SOCKET_HEIGHT;

            socketPositionMapping[socketKey] = [
                (socket[1] === SocketDirection.Output ? width : 0),
                y + SOCKET_OFFSET
            ];
        }

        exportSocketPositionMapping(socketPositionMapping, wrapper.index.toKey());
    }
    
    export let onMousedown = function(index: NodeIndex, e: MouseEvent) {};
    export let onSocketMousedown = function(event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection, index: NodeIndex) {};
    export let onSocketMouseup = function(event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection, index: NodeIndex) {};

    function onMousedownRaw (e: MouseEvent) {
        onMousedown(new NodeIndex(wrapper.index.index, wrapper.index.generation), e);
    }

    function onSocketMousedownRaw (event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection) {
        onSocketMousedown(event, socket, direction, wrapper.index);
    }

    function onSocketMouseupRaw (event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection) {
        onSocketMouseup(event, socket, direction, wrapper.index);
    }

    let x, y, selected, title;

    $: x = wrapper.uiData.x;
    $: y = wrapper.uiData.y;
    $: selected = wrapper.uiData.selected;
    $: title = wrapper.uiData.title;
</script>

<div class="background" style="transform: translate({x}px, {y}px); width: {width}px" on:mousedown={onMousedownRaw} class:selected={selected} bind:this={node}>
    <div class="node-title">{title && title.length > 0 ? title : " "}</div>

    {#each sockets as [socket, direction, def] (socketToKey(socket, direction))}
        <NodeRowUI
            type={socket}
            direction={direction}
            label={socketTypeToString(socket)}
            defaultValue={def}
            socketMousedown={onSocketMousedownRaw}
            socketMouseup={onSocketMouseupRaw}
            nodeWrapper={wrapper}
        />
    {/each}
</div>

<style>
.node-title {
    color: white;
    font-size: 18px;
    min-height: 22px;
    margin: 8px;
    text-overflow: ellipsis;
    white-space: nowrap;
    overflow: hidden;
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
    background-color: rgba(148, 195, 255, 0.8);
}

.right-align {
    text-anchor: end;
}

.title {
    font-size: 18px;
}
</style>