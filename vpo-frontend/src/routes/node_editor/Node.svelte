<script lang="ts">
    import NodePropertyRow from "./NodePropertyRow.svelte";
    import { createEventDispatcher } from "svelte";
    import type { OverrideUpdateEvent, SocketEvent } from "./socket";
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import type { NodeWrapper, NodeRow } from "$lib/node-engine/node";
    import {
        match,
        type DiscriminatedUnion,
        matchOrElse,
    } from "$lib/util/discriminated-union";
    import type { Property, PropertyType } from "$lib/node-engine/property";
    import { localize } from "@nubolab-ffwd/svelte-fluent";
    import type { SocketRegistry } from "$lib/node-engine/socket_registry";
    import type {
        Socket,
        SocketDirection,
        SocketType,
        SocketValue,
    } from "$lib/node-engine/connection";
    import type { VertexIndex } from "$lib/ddgg/graph";
    import UiNodeRow from "./UiNodeRow.svelte";
    import { deepEqual } from "fast-equals";
    // in pixels, these numbers are derived from the css below and the css in ./Socket.svelte
    // update in node-engine/node.ts, constants at the top

    export let width = 270;

    export let nodes: NodeGraph;
    export let wrapper: NodeWrapper;
    export let nodeIndex: VertexIndex;
    export let registry: SocketRegistry;

    const dispatch = createEventDispatcher();

    type ReducedRowType = DiscriminatedUnion<
        "variant",
        {
            SocketRow: {
                socket: Socket;
                socketDirection: SocketDirection;
                value: SocketValue;
            };
            PropertyRow: {
                propName: string;
                propType: PropertyType;
                propValue: Property;
            };
            InnerGraphRow: {};
        }
    >;

    let sockets: ReducedRowType[] = [];
    $: sockets = wrapper.nodeRows.map((nodeRow) =>
        match(nodeRow, {
            Input: ({ data: [socket, defaultValue] }): ReducedRowType => ({
                variant: "SocketRow",
                socket,
                socketDirection: { variant: "Input" },
                value: nodes.getNodeSocketDefault(nodeIndex, socket),
            }),
            Output: ({ data: socket }) => ({
                variant: "SocketRow",
                socket,
                socketDirection: { variant: "Output" },
                value: { variant: "None" },
            }),
            Property: ({ data: [propName, propType, _defaultValue] }) => ({
                variant: "PropertyRow",
                propName,
                propType,
                propValue: nodes.getNodePropertyValue(nodeIndex, propName) ?? {
                    variant: "String",
                    data: "",
                },
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
            ? JSON.stringify([row.socket, row.socketDirection])
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
                        ? $localize("node." + wrapper.uiData.title)
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

    function onOverrideUpdate(event: CustomEvent<OverrideUpdateEvent>) {
        const index = wrapper.defaultOverrides.findIndex((row) => {
            return matchOrElse(
                row,
                {
                    Input: ({ data: [socket] }) => {
                        return (
                            event.detail.direction.variant === "Input" &&
                            deepEqual(event.detail.socket, socket)
                        );
                    },
                    Output: ({ data: socket }) => {
                        return (
                            event.detail.direction.variant === "Output" &&
                            deepEqual(event.detail.socket, socket)
                        );
                    },
                },
                () => false
            );
        });

        if (index !== -1) {
            wrapper.defaultOverrides[index] = {
                variant: "Input",
                data: [event.detail.socket, event.detail.newValue],
            };
        } else {
            wrapper.defaultOverrides.push({
                variant: "Input",
                data: [event.detail.socket, event.detail.newValue],
            });
        }

        nodes.ipcSocket.send({
            action: "graph/updateNodes",
            payload: {
                graphIndex: nodes.graphIndex,
                updatedNodes: [
                    [
                        {
                            defaultOverrides:
                                nodes.getNode(nodeIndex)?.defaultOverrides,
                        },
                        nodeIndex,
                    ],
                ],
            },
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
    on:dblclick|stopPropagation
>
    <div class="node-title">
        {wrapper.uiData.title && wrapper.uiData.title.length > 0
            ? $localize("node." + wrapper.uiData.title)
            : " "}
    </div>

    {#each sockets as row (rowToKey(row))}
        {#if row.variant === "SocketRow"}
            <UiNodeRow
                {nodes}
                socket={row.socket}
                direction={row.socketDirection}
                label={$localize(
                    ...registry.getSocketInterpolation(row.socket)
                )}
                on:socketMousedown={onSocketMousedown}
                on:socketMouseup={onSocketMouseup}
                on:overrideUpdate={onOverrideUpdate}
                value={row.value}
                {nodeIndex}
            />
        {:else if row.variant === "PropertyRow"}
            <NodePropertyRow
                {nodes}
                nodeWrapper={wrapper}
                propName={row.propName}
                propType={row.propType}
                value={row.propValue}
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
