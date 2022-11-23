<script lang="ts">
    import NodeRowUI from "./NodeRow.svelte";
    import { NodeIndex } from "../node-engine/node_index";
    import type { NodeWrapper, SocketValue } from "../node-engine/node";
    import type { NodeGraph } from "../node-engine/node_graph";
    import {
        SocketType,
        SocketDirection,
        socketToKey,
    } from "../node-engine/connection";
    import { socketTypeToString } from "./interpolation";
    import NodePropertyRow from "./NodePropertyRow.svelte";
    import { i18n } from "../i18n.js";
    import { Property, PropertyType } from "../node-engine/property";
    import { createEventDispatcher } from "svelte";
    import { DiscriminatedUnion, match } from "../util/discriminated-union";
    import type { SocketEvent } from "./socket";

    // in pixels, these numbers are derived from the css below and the css in ./Socket.svelte
    // update in node-engine/node.ts, constants at the top

    export let width = 270;

    export let nodes: NodeGraph;
    export let wrapper: NodeWrapper;

    const dispatch = createEventDispatcher();

    type ReducedRowType = DiscriminatedUnion<
        "variant",
        {
            SocketRow: {
                socketType: SocketType;
                socketDirection: SocketDirection;
                value: SocketValue;
                hidden: boolean;
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
    $: sockets = wrapper.node_rows.map((nodeRow) =>
        match(nodeRow, {
            StreamInput: ({
                data: [streamInput, defaultValue, hidden],
            }): ReducedRowType => ({
                variant: "SocketRow",
                socketType: { variant: "Stream", data: streamInput },
                socketDirection: SocketDirection.Input,
                value: { variant: "Stream", data: defaultValue },
                hidden,
            }),
            MidiInput: ({ data: [midiInput, defaultValue, hidden] }) => ({
                variant: "SocketRow",
                socketType: { variant: "Midi", data: midiInput },
                socketDirection: SocketDirection.Input,
                value: { variant: "Midi", data: defaultValue },
                hidden,
            }),
            ValueInput: ({ data: [valueInput, defaultValue, hidden] }) => ({
                variant: "SocketRow",
                socketType: { variant: "Value", data: valueInput },
                socketDirection: SocketDirection.Input,
                value: { variant: "Primitive", data: defaultValue },
                hidden,
            }),
            NodeRefInput: ({ data: [nodeRefInput, hidden] }) => ({
                variant: "SocketRow",
                socketType: { variant: "NodeRef", data: nodeRefInput },
                socketDirection: SocketDirection.Input,
                value: { variant: "None" },
                hidden,
            }),
            StreamOutput: ({ data: [streamOutput, defaultValue, hidden] }) => ({
                variant: "SocketRow",
                socketType: { variant: "Stream", data: streamOutput },
                socketDirection: SocketDirection.Output,
                value: { variant: "Stream", data: defaultValue },
                hidden,
            }),
            MidiOutput: ({ data: [midiOutput, defaultValue, hidden] }) => ({
                variant: "SocketRow",
                socketType: { variant: "Midi", data: midiOutput },
                socketDirection: SocketDirection.Output,
                value: { variant: "Midi", data: defaultValue },
                hidden,
            }),
            ValueOutput: ({ data: [valueOutput, defaultValue, hidden] }) => ({
                variant: "SocketRow",
                socketType: { variant: "Value", data: valueOutput },
                socketDirection: SocketDirection.Output,
                value: { variant: "Primitive", data: defaultValue },
                hidden,
            }),
            NodeRefOutput: ({ data: [nodeRefOutput, hidden] }) => ({
                variant: "SocketRow",
                socketType: { variant: "NodeRef", data: nodeRefOutput },
                socketDirection: SocketDirection.Output,
                value: { variant: "None" },
                hidden,
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

    export let onMousedown = function (index: NodeIndex, e: MouseEvent) {};

    function onMousedownRaw(e: MouseEvent) {
        onMousedown(wrapper.index, e);
    }

    function rowToKey(row: ReducedRowType): string {
        return row.variant === "SocketRow"
            ? socketToKey(row.socketType, row.socketDirection)
            : row.variant === "PropertyRow"
            ? "prop." + row.propName
            : "innerGraph";
    }

    function openInnerGraph() {
        if (wrapper.child_graph_index !== null) {
            dispatch("changeGraph", {
                graphIndex: wrapper.child_graph_index,
                nodeTitle:
                    wrapper.ui_data.title && wrapper.ui_data.title.length > 0
                        ? i18n.t("nodes." + wrapper.ui_data.title)
                        : " ",
            });
        }
    }

    function onSocketMousedown(event: CustomEvent<SocketEvent>) {
        dispatch("socketMousedown", {
            ...event.detail,
            nodeIndex: { ...wrapper.index },
        });
    }

    function onSocketMouseup(event: CustomEvent<SocketEvent>) {
        dispatch("socketMouseup", {
            ...event.detail,
            nodeIndex: { ...wrapper.index },
        });
    }
</script>

<div
    class="background"
    style="transform: translate({wrapper.ui_data.x}px, {wrapper.ui_data
        .y}px); width: {width}px"
    on:mousedown={onMousedownRaw}
    class:selected={wrapper.ui_data.selected}
    bind:this={node}
>
    <div class="node-title">
        {wrapper.ui_data.title && wrapper.ui_data.title.length > 0
            ? i18n.t("nodes." + wrapper.ui_data.title)
            : " "}
    </div>

    {#each sockets as row (rowToKey(row))}
        {#if row.variant === "SocketRow"}
            {#if !row.hidden}
                <NodeRowUI
                    {nodes}
                    type={row.socketType}
                    direction={row.socketDirection}
                    label={socketTypeToString(row.socketType)}
                    hidden={row.hidden}
                    on:socketMousedown={onSocketMousedown}
                    on:socketMouseup={onSocketMouseup}
                    nodeWrapper={wrapper}
                />
            {/if}
        {:else if row.variant === "PropertyRow"}
            <NodePropertyRow
                {nodes}
                nodeWrapper={wrapper}
                propName={row.propName}
                propType={row.propType}
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
