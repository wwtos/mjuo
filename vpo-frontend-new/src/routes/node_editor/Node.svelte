<script lang="ts">
    import NodeRowUI from "./NodeRow.svelte";
    import { socketTypeToString } from "./interpolation";
    import NodePropertyRow from "./NodePropertyRow.svelte";
    import { createEventDispatcher } from "svelte";
    import type { SocketEvent } from "./socket";
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import type { NodeWrapper, SocketValue } from "$lib/node-engine/node";
    import type { VertexIndex } from "$lib/ddgg/graph";
    import {
        match,
        type DiscriminatedUnion,
    } from "$lib/util/discriminated-union";
    import {
        SocketDirection,
        socketToKey,
        type SocketType,
    } from "$lib/node-engine/connection";
    import type { Property, PropertyType } from "$lib/node-engine/property";
    import { localize } from "@nubolab-ffwd/svelte-fluent";
    import type { SocketRegistry } from "$lib/node-engine/socket_registry";

    // in pixels, these numbers are derived from the css below and the css in ./Socket.svelte
    // update in node-engine/node.ts, constants at the top

    export let width = 270;

    export let nodes: NodeGraph;
    export let wrapper: NodeWrapper;
    export let nodeIndex: VertexIndex;
    export let socketRegistry: SocketRegistry;

    const dispatch = createEventDispatcher();

    type ReducedRowType = DiscriminatedUnion<
        "variant",
        {
            SocketRow: {
                socketType: SocketType;
                socketDirection: SocketDirection;
                value: SocketValue;
                polyphonic: boolean;
            };
            PropertyRow: {
                propName: string;
                propType: PropertyType;
                defaultValue: Property;
            };
            InnerGraphRow: {};
        }
    >;

    let sockets: ReducedRowType[] = [];
    $: sockets = wrapper.nodeRows.map((nodeRow) =>
        match(nodeRow, {
            StreamInput: ({
                data: [streamInput, defaultValue, polyphonic],
            }): ReducedRowType => ({
                variant: "SocketRow",
                socketType: { variant: "Stream", data: streamInput },
                socketDirection: SocketDirection.Input,
                value: { variant: "Stream", data: defaultValue },
                polyphonic,
            }),
            MidiInput: ({ data: [midiInput, defaultValue, polyphonic] }) => ({
                variant: "SocketRow",
                socketType: { variant: "Midi", data: midiInput },
                socketDirection: SocketDirection.Input,
                value: { variant: "Midi", data: defaultValue },
                polyphonic,
            }),
            ValueInput: ({ data: [valueInput, defaultValue, polyphonic] }) => ({
                variant: "SocketRow",
                socketType: { variant: "Value", data: valueInput },
                socketDirection: SocketDirection.Input,
                value: { variant: "Primitive", data: defaultValue },
                polyphonic,
            }),
            NodeRefInput: ({ data: [nodeRefInput, polyphonic] }) => ({
                variant: "SocketRow",
                socketType: { variant: "NodeRef", data: nodeRefInput },
                socketDirection: SocketDirection.Input,
                value: { variant: "None" },
                polyphonic,
            }),
            StreamOutput: ({
                data: [streamOutput, defaultValue, polyphonic],
            }) => ({
                variant: "SocketRow",
                socketType: { variant: "Stream", data: streamOutput },
                socketDirection: SocketDirection.Output,
                value: { variant: "Stream", data: defaultValue },
                polyphonic,
            }),
            MidiOutput: ({ data: [midiOutput, defaultValue, polyphonic] }) => ({
                variant: "SocketRow",
                socketType: { variant: "Midi", data: midiOutput },
                socketDirection: SocketDirection.Output,
                value: { variant: "Midi", data: defaultValue },
                polyphonic,
            }),
            ValueOutput: ({
                data: [valueOutput, defaultValue, polyphonic],
            }) => ({
                variant: "SocketRow",
                socketType: { variant: "Value", data: valueOutput },
                socketDirection: SocketDirection.Output,
                value: { variant: "Primitive", data: defaultValue },
                polyphonic,
            }),
            NodeRefOutput: ({ data: [nodeRefOutput, polyphonic] }) => ({
                variant: "SocketRow",
                socketType: { variant: "NodeRef", data: nodeRefOutput },
                socketDirection: SocketDirection.Output,
                value: { variant: "None" },
                polyphonic,
            }),
            Property: ({ data: [propName, propType, defaultValue] }) => ({
                variant: "PropertyRow",
                propName,
                propType,
                defaultValue,
            }),
            InnerGraph: () => ({ variant: "InnerGraphRow" }),
        })
    );

    let node: HTMLDivElement;

    export let onMousedown = function (index: VertexIndex, e: MouseEvent) {};

    function onMousedownRaw(e: MouseEvent) {
        onMousedown(nodeIndex, e);
    }

    function rowToKey(row: ReducedRowType): string {
        return row.variant === "SocketRow"
            ? socketToKey(row.socketType, row.socketDirection)
            : row.variant === "PropertyRow"
            ? "prop." + row.propName
            : "innerGraph";
    }

    function openInnerGraph() {
        if (wrapper.childGraphIndex !== null) {
            dispatch("changeGraph", {
                graphIndex: wrapper.childGraphIndex,
                nodeTitle:
                    wrapper.uiData.title && wrapper.uiData.title.length > 0
                        ? $localize("node-" + wrapper.uiData.title)
                        : " ",
            });
        }
    }

    function onSocketMousedown(event: CustomEvent<SocketEvent>) {
        dispatch("socketMousedown", {
            ...event.detail,
            vertexIndex: { ...nodeIndex },
        });
    }

    function onSocketMouseup(event: CustomEvent<SocketEvent>) {
        dispatch("socketMouseup", {
            ...event.detail,
            vertexIndex: { ...nodeIndex },
        });
    }
</script>

<div
    class="background"
    style="transform: translate({wrapper.uiData.x}px, {wrapper.uiData
        .y}px); width: {width}px"
    on:mousedown={onMousedownRaw}
    class:selected={wrapper.uiData.selected}
    bind:this={node}
>
    <div class="node-title">
        {wrapper.uiData.title && wrapper.uiData.title.length > 0
            ? $localize("node-" + wrapper.uiData.title)
            : " "}
    </div>

    {#each sockets as row (rowToKey(row))}
        {#if row.variant === "SocketRow"}
            <NodeRowUI
                {nodes}
                type={row.socketType}
                direction={row.socketDirection}
                label={socketTypeToString(
                    socketRegistry,
                    row.socketType,
                    $localize
                )}
                polyphonic={row.polyphonic}
                on:socketMousedown={onSocketMousedown}
                on:socketMouseup={onSocketMouseup}
                nodeWrapper={wrapper}
                {nodeIndex}
            />
        {:else if row.variant === "PropertyRow"}
            <NodePropertyRow
                {nodes}
                nodeWrapper={wrapper}
                propName={row.propName}
                propType={row.propType}
                {nodeIndex}
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
</style>
