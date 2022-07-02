<script lang="ts">
    import NodeRowUI from "./NodeRow.svelte";
    import { NodeIndex, NodeRow, NodeWrapper } from "../node-engine/node";
    import { Graph } from "../node-engine/graph";
    import { SocketType, SocketDirection, socketTypeToString, socketToKey } from "../node-engine/connection";
    import { map } from "rxjs/operators";
    import NodePropertyRow from "./NodePropertyRow.svelte";
    import { MemberType } from "safety-match";
    import { i18n } from '../i18n.js';

    // in pixels, these numbers are derived from the css below and the css in ./Socket.svelte
    // update in node-engine/node.ts, constants at the top

    export let width = 200;

    export let nodes: Graph;
    export let wrapper: NodeWrapper;

    let sockets = wrapper.nodeRows.pipe(
        map(nodeRows => {
            return nodeRows.map(nodeRow => {
                return nodeRow.match({
                    StreamInput: ([streamInput, def]) => [SocketType.Stream(streamInput), SocketDirection.Input, def],
                    MidiInput: ([midiInput, def]) => [SocketType.Midi(midiInput), SocketDirection.Input, def],
                    ValueInput: ([valueInput, def]) => [SocketType.Value(valueInput), SocketDirection.Input, def],
                    NodeRefInput: (nodeRefInput) => [SocketType.NodeRef(nodeRefInput), SocketDirection.Input, undefined],
                    StreamOutput: ([streamOutput, def]) => [SocketType.Stream(streamOutput), SocketDirection.Output, def],
                    MidiOutput: ([midiOutput, def]) => [SocketType.Midi(midiOutput), SocketDirection.Output, def],
                    ValueOutput: ([valueOutput, def]) => [SocketType.Value(valueOutput), SocketDirection.Output, def],
                    NodeRefOutput: (nodeRefOutput) => [SocketType.NodeRef(nodeRefOutput), SocketDirection.Output, undefined],
                    Property: ([propName, propType, propDefault]) => {
                        return ["property", propName, propType, propDefault];
                    },
                    _: () => {}
                });
            }).filter(maybeSomething => !!maybeSomething);
        })
    );

    let node: HTMLDivElement;
    
    export let onMousedown = function(index: NodeIndex, e: MouseEvent) {};
    export let onSocketMousedown = function(event: MouseEvent, socket: MemberType<typeof SocketType>, direction: SocketDirection, index: NodeIndex) {};
    export let onSocketMouseup = function(event: MouseEvent, socket: MemberType<typeof SocketType>, direction: SocketDirection, index: NodeIndex) {};

    function onMousedownRaw (e: MouseEvent) {
        onMousedown(new NodeIndex(wrapper.index.index, wrapper.index.generation), e);
    }

    function onSocketMousedownRaw (event: MouseEvent, socket: MemberType<typeof SocketType>, direction: SocketDirection) {
        onSocketMousedown(event, socket, direction, wrapper.index);
    }

    function onSocketMouseupRaw (event: MouseEvent, socket: MemberType<typeof SocketType>, direction: SocketDirection) {
        onSocketMouseup(event, socket, direction, wrapper.index);
    }

    const uiData = wrapper.uiData;

    function rowToKey(row): string {
        return row[0] !== "property" ? socketToKey(row[0], row[1]) : ("prop" + row[1]);
    }
</script>

<div class="background" style="transform: translate({$uiData.x}px, {$uiData.y}px); width: {width}px" on:mousedown={onMousedownRaw} class:selected={$uiData.selected} bind:this={node}>
    <div class="node-title">{$uiData.title && $uiData.title.length > 0 ? i18n.t("nodes." + $uiData.title) : " "}</div>

    {#each $sockets as row (rowToKey(row))}
        {#if row[0] !== "property"}
            <NodeRowUI
                {nodes}
                type={row[0]}
                direction={row[1]}
                label={socketTypeToString(row[0])}
                defaultValue={row[2]}
                socketMousedown={onSocketMousedownRaw}
                socketMouseup={onSocketMouseupRaw}
                nodeWrapper={wrapper}
            />
        {:else}
            <NodePropertyRow
                {nodes}
                nodeWrapper={wrapper}
                propName={row[1]}
                propType={row[2]}
            />
        {/if}
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
    border: solid 2px #84b8e9;
}

.right-align {
    text-anchor: end;
}

.title {
    font-size: 18px;
}
</style>