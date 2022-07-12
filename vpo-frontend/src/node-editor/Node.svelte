<script lang="ts">
    import NodeRowUI from "./NodeRow.svelte";
    import { NodeIndex, NodeRow, NodeWrapper } from "../node-engine/node";
    import { NodeGraph } from "../node-engine/node_graph";
    import { SocketType, SocketDirection, socketToKey } from "../node-engine/connection";
    import { socketTypeToString } from "./interpolation";
    import { map } from "rxjs/operators";
    import NodePropertyRow from "./NodePropertyRow.svelte";
    import { makeTaggedUnion, MemberType, none } from "safety-match";
    import { i18n } from '../i18n.js';
    import { Property, PropertyType } from "../node-engine/property";
    import { IPCSocket } from "../util/socket";

    // in pixels, these numbers are derived from the css below and the css in ./Socket.svelte
    // update in node-engine/node.ts, constants at the top

    export let width = 200;

    export let nodes: NodeGraph;
    export let wrapper: NodeWrapper;
    export let ipcSocket: IPCSocket;

    const ReducedRowType = makeTaggedUnion({
        SocketRow(socketType: MemberType<typeof SocketType>, socketDirection: SocketDirection, value: any) {
            return [socketType, socketDirection, value];
        },
        PropertyRow(propName: string, propType: MemberType<typeof PropertyType>, defaultValue: MemberType<typeof Property>) {
            return [propName, propType, defaultValue];
        },
        InnerGraphRow: none
    });

    let sockets = wrapper.nodeRows.pipe(
        map((nodeRows) => {
            return nodeRows.map(nodeRow => {
                return nodeRow.match({
                    StreamInput: ([streamInput, def]) => ReducedRowType.SocketRow(
                        SocketType.Stream(streamInput), SocketDirection.Input, def
                    ),
                    MidiInput: ([midiInput, def]) => ReducedRowType.SocketRow(
                        SocketType.Midi(midiInput), SocketDirection.Input, def
                    ),
                    ValueInput: ([valueInput, def]) => ReducedRowType.SocketRow(
                        SocketType.Value(valueInput), SocketDirection.Input, def
                    ),
                    NodeRefInput: (nodeRefInput) => ReducedRowType.SocketRow(
                        SocketType.NodeRef(nodeRefInput), SocketDirection.Input, undefined
                    ),
                    StreamOutput: ([streamOutput, def]) => ReducedRowType.SocketRow(
                        SocketType.Stream(streamOutput), SocketDirection.Output, def
                    ),
                    MidiOutput: ([midiOutput, def]) => ReducedRowType.SocketRow(
                        SocketType.Midi(midiOutput), SocketDirection.Output, def
                    ),
                    ValueOutput: ([valueOutput, def]) => ReducedRowType.SocketRow(
                        SocketType.Value(valueOutput), SocketDirection.Output, def
                    ),
                    NodeRefOutput: (nodeRefOutput) => ReducedRowType.SocketRow(
                        SocketType.NodeRef(nodeRefOutput), SocketDirection.Output, undefined
                    ),
                    Property: ([propName, propType, propDefault]) => {
                        return ReducedRowType.PropertyRow(
                            propName, propType, propDefault
                        );
                    },
                    InnerGraph: () => {
                        return ReducedRowType.InnerGraphRow;
                    }
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

    function rowToKey(row: MemberType<typeof ReducedRowType>): string {
        return row.variant === "SocketRow" ? socketToKey(row.data[0], row.data[1]) :
               row.variant === "PropertyRow" ? "prop." + row.data[0] :
               "innerGraph";
    }

    function openInnerGraph() {
        if (wrapper.innerGraphIndex !== null) {
            ipcSocket.switchToGraph(wrapper.innerGraphIndex);
        }
    }
</script>

<div class="background" style="transform: translate({$uiData.x}px, {$uiData.y}px); width: {width}px" on:mousedown={onMousedownRaw} class:selected={$uiData.selected} bind:this={node}>
    <div class="node-title">{$uiData.title && $uiData.title.length > 0 ? i18n.t("nodes." + $uiData.title) : " "}</div>

    {#each $sockets as row (rowToKey(row))}
        {#if row.variant === "SocketRow" }
            <NodeRowUI
                {nodes}
                type={row.data[0]}
                direction={row.data[1]}
                label={socketTypeToString(row.data[0])}
                defaultValue={row.data[2]}
                socketMousedown={onSocketMousedownRaw}
                socketMouseup={onSocketMouseupRaw}
                nodeWrapper={wrapper}
            />
        {:else if row.variant === "PropertyRow"}
            <NodePropertyRow
                {nodes}
                nodeWrapper={wrapper}
                propName={row.data[0]}
                propType={row.data[1]}
            />
        {:else}
            <div class="container">
                <button on:click={openInnerGraph}>Open inner graph</button>
            </div>
        {/if}
    {/each}
</div>

<style>
button {
    width: calc(100% - 32px);
    margin: 0 16px;
    height: 26px;
}

.container {
    margin: 10px 0;
    height: 26px;
}

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