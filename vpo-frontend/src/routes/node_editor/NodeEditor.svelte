<script lang="ts">
    import Node from "./Node.svelte";
    import ConnectionUI from "./Connection.svelte";
    import { onMount } from "svelte";
    import panzoom, { type PanZoom } from "panzoom";
    import type { BehaviorSubject, Subscription } from "rxjs";
    import type { SocketEvent } from "./socket";
    import NodeCreationMenu from "./NodeCreationMenu.svelte";
    import Breadcrumb from "./Breadcrumb.svelte";
    import { deselectAll, getSelected } from "./editor-utils";
    import type { IpcSocket } from "$lib/ipc/socket";
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import type { VertexIndex } from "$lib/ddgg/graph";
    import { transformMouseRelativeToEditor } from "$lib/util/mouse-transforms";
    import { NODE_WIDTH, type NodeInstance } from "$lib/node-engine/node";
    import type { GraphManager } from "$lib/node-engine/graph_manager";
    import type {
        Socket,
        SocketDirection,
        Connection,
    } from "$lib/node-engine/connection";
    import type { Action } from "$lib/node-engine/state";
    import { deepEqual } from "fast-equals";

    export let width = 400;
    export let height = 400;

    export let ipcSocket: IpcSocket;
    export let activeGraph: BehaviorSubject<NodeGraph>;
    export let graphManager: GraphManager;

    let previousSubscriptions: Array<Subscription> = [];

    let nodeTypeToCreate: string;

    let zoomer: PanZoom;

    ipcSocket.requestGraph("0.0");

    let editor: HTMLDivElement;
    let nodeContainer: HTMLDivElement;

    // nodes being actively dragged as well as their mouse offset
    let draggedState: Array<{
        node: VertexIndex;
        offset: [number, number];
    }> = [];
    let keepSelected: VertexIndex | null = null;

    let createNodeMenu = {
        visible: false,
        x: 0,
        y: 0,
    };

    // the points of the connection being created
    let connectionBeingCreated: null | {
        x1: number;
        y1: number;
        x2: number;
        y2: number;
    } = null;

    let connectionBeingCreatedFrom: {
        index: VertexIndex;
        direction: SocketDirection;
        socket: Socket;
    };

    let selection: {
        fromClientX: number;
        fromClientY: number;
        fromX: number;
        fromY: number;
        toX: number;
        toY: number;
    } | null = null;

    let path = [
        {
            name: "root",
            index: "0.0",
        },
    ];

    let controlHeld = false;
    let shiftHeld = false;

    function onPaste(event: ClipboardEvent) {
        const paste = event.clipboardData?.getData("text") || "";

        deselectAll($activeGraph);
        $activeGraph.writeChangedNodesToServer();

        ipcSocket.paste($activeGraph.graphIndex, paste);
    }

    function onKeydown(event: KeyboardEvent) {
        controlHeld = event.ctrlKey;
        shiftHeld = event.shiftKey;

        if (event.ctrlKey) {
            switch (event.key) {
                case "c":
                    ipcSocket.copy($activeGraph.graphIndex);
                    break;
            }
        } else {
            switch (event.key) {
                case "Delete":
                    const selected = $activeGraph.nodeStore
                        .getValue()
                        .filter(([{ data: node }, _]) => node?.uiData.selected)
                        .map(([_, index]) => index);

                    const commits: Action[] = selected.map((index) => ({
                        variant: "RemoveNode",
                        data: {
                            index: {
                                graphIndex: $activeGraph.graphIndex,
                                nodeIndex: index,
                            },
                        },
                    }));

                    ipcSocket.commit(commits);

                    break;
            }
        }
    }

    function onKeyup(event: KeyboardEvent) {
        controlHeld = event.ctrlKey;
        shiftHeld = event.shiftKey;
    }

    function onNodeMousedown(index: VertexIndex, event: MouseEvent) {
        if (event.button === 0) {
            // translate mouse coordinates

            zoomer.pause();

            keepSelected = index;
        }
    }

    function onEditorMousedown(event: MouseEvent) {
        let [mouseX, mouseY] = transformMouseRelativeToEditor(
            editor,
            zoomer,
            event.clientX,
            event.clientY,
        );

        if (shiftHeld) {
            zoomer.pause();

            event.preventDefault();

            selection = {
                fromClientX: event.clientX,
                fromClientY: event.clientY,
                fromX: mouseX,
                fromY: mouseY,
                toX: mouseX,
                toY: mouseY,
            };
        }
    }

    function onMousemove({ clientX, clientY }: MouseEvent) {
        // convert window coordinates to editor coordinates
        let [mouseX, mouseY] = transformMouseRelativeToEditor(
            editor,
            zoomer,
            clientX,
            clientY,
        );

        if (selection) {
            selection.toX = mouseX;
            selection.toY = mouseY;
        }

        // have we not started dragging?
        if (draggedState.length === 0 && keepSelected) {
            const node = $activeGraph.getNode(keepSelected) as NodeInstance;

            // if control isn't pressed, and the one clicked isn't selected, deselect all of them
            if (!controlHeld && !node.uiData.selected) {
                deselectAll($activeGraph);
            }

            // mark this one as dragging
            node.uiData = {
                ...node.uiData,
                selected: true,
            };

            $activeGraph.markNodeAsUpdated(keepSelected, ["uiData"]);

            // figure out what nodes should be dragged
            const selected = getSelected($activeGraph);

            draggedState = selected.map((nodeIndex) => {
                const node = $activeGraph.getNode(nodeIndex) as NodeInstance;
                const offset: [number, number] = [
                    mouseX - node.uiData.x,
                    mouseY - node.uiData.y,
                ];

                return {
                    node: nodeIndex,
                    offset,
                };
            });
        }

        // if the mouse was moved and we are dragging nodes, update those node's position
        if (draggedState.length > 0) {
            for (let { node: nodeIndex, offset } of draggedState) {
                let node = $activeGraph.getNode(nodeIndex) as NodeInstance;

                node.uiData = {
                    ...node.uiData,
                    x: mouseX - offset[0],
                    y: mouseY - offset[1],
                };

                $activeGraph.markNodeAsUpdated(nodeIndex, ["uiData"]);
                $activeGraph.update();
            }
        } else if (connectionBeingCreated) {
            connectionBeingCreated.x2 = mouseX;
            connectionBeingCreated.y2 = mouseY;
        }
    }

    function onContextMenu(event: MouseEvent) {
        createNodeMenu.visible = true;
        createNodeMenu.x = event.clientX - 125;
        createNodeMenu.y = event.clientY;

        return false;
    }

    const onWindowMousedown = (event: MouseEvent) => {
        createNodeMenu.visible = false;
    };

    function onMouseup(event: MouseEvent) {
        let [mouseX, mouseY] = transformMouseRelativeToEditor(
            editor,
            zoomer,
            event.clientX,
            event.clientY,
        );

        if (draggedState.length === 0 && !controlHeld) {
            let selectedNodes = getSelected($activeGraph);

            for (let selected of selectedNodes) {
                if (!deepEqual(selected, keepSelected)) {
                    const node = $activeGraph.getNode(selected) as NodeInstance;

                    node.uiData.selected = false;
                }

                $activeGraph.markNodeAsUpdated(selected, ["uiData"]);
            }
        }

        if (keepSelected) {
            // mark this one as dragging
            const node = $activeGraph.getNode(keepSelected) as NodeInstance;

            if (!node.uiData.selected) {
                node.uiData = {
                    ...node.uiData,
                    selected: true,
                };

                $activeGraph.markNodeAsUpdated(keepSelected, ["uiData"]);
            }
        }

        if (selection) {
            const AABB = {
                left: selection.fromClientX,
                top: selection.fromClientY,
                right: event.clientX,
                bottom: event.clientY,
            };

            const nodes = Array.from(document.querySelectorAll(".node"));

            const insideAABB = nodes
                .filter((node) => {
                    const nodeAABB = node.getBoundingClientRect();

                    return (
                        AABB.right >= nodeAABB.left &&
                        nodeAABB.right >= AABB.left &&
                        AABB.bottom >= nodeAABB.top &&
                        nodeAABB.bottom >= AABB.top
                    );
                })
                .map((node) => {
                    let index = node.getAttribute("data-index") || "";

                    return index;
                });

            deselectAll($activeGraph);

            for (let index of insideAABB) {
                const node = $activeGraph.getNode(index) as NodeInstance;

                node.uiData.selected = true;

                $activeGraph.markNodeAsUpdated(index, ["uiData"]);
            }

            $activeGraph.writeChangedNodesToServer();
        }

        zoomer.resume();

        selection = null;
        draggedState = [];
        connectionBeingCreated = null;
        keepSelected = null;

        $activeGraph.writeChangedNodesToServer();
    }

    function createNode(
        ev: CustomEvent<{
            value: string;
            clientX: number;
            clientY: number;
        }>,
    ) {
        let [mouseX, mouseY] = transformMouseRelativeToEditor(
            editor,
            zoomer,
            ev.detail.clientX,
            ev.detail.clientY,
        );

        ipcSocket.commit([
            {
                variant: "CreateNode",
                data: {
                    graph: $activeGraph.graphIndex,
                    nodeType: ev.detail.value,
                    uiData: {
                        x: mouseX - NODE_WIDTH / 2,
                        y: mouseY - 30,
                    },
                },
            },
        ]);

        createNodeMenu.visible = false;
    }

    let keyedNodes: [string, NodeInstance, VertexIndex][];
    let keyedConnections: [string, Connection][];

    $: {
        if (previousSubscriptions.length > 0) {
            previousSubscriptions.forEach((sub) => sub.unsubscribe());
        }

        previousSubscriptions = [
            $activeGraph.keyedNodeStore.subscribe((newKeyedNodes) => {
                keyedNodes = newKeyedNodes;
            }),
            $activeGraph.keyedConnectionStore.subscribe(
                (newKeyedConnections) => {
                    keyedConnections = newKeyedConnections;
                },
            ),
        ];
    }

    (window as any)["ipcSocket"] = ipcSocket;

    function handleSocketMousedown(event: CustomEvent<SocketEvent>) {
        let e = event.detail;

        let [mouseX, mouseY] = transformMouseRelativeToEditor(
            editor,
            zoomer,
            e.event.clientX,
            e.event.clientY,
        );

        if (e.direction.variant === "Input") {
            // see if it's already connected, in which case we're disconnecting it
            let connection = $activeGraph.getNodeInputConnection(
                e.vertexIndex,
                e.socket,
            );

            // check if we are already connected
            if (connection) {
                ipcSocket.commit(
                    [
                        {
                            variant: "DisconnectNodes",
                            data: {
                                from: connection.fromNode,
                                to: e.vertexIndex,
                                data: {
                                    fromSocket: connection.fromSocket,
                                    toSocket: connection.toSocket,
                                },
                                graph: $activeGraph.graphIndex,
                            },
                        },
                    ],
                    false,
                );

                // add the connection line back for connecting to something else
                connectionBeingCreatedFrom = {
                    index: connection.fromNode,
                    direction: { variant: "Output" },
                    socket: connection.fromSocket,
                };

                const fromNode = $activeGraph.getNode(connection.fromNode);
                const fromNodeXY = $activeGraph.getNodeSocketXy(
                    connection.fromNode,
                    connection.fromSocket,
                    { variant: "Output" },
                );

                connectionBeingCreated = {
                    x1: fromNodeXY.x,
                    y1: fromNodeXY.y,
                    x2: mouseX,
                    y2: mouseY,
                };

                return;
            }
        }

        connectionBeingCreatedFrom = {
            index: e.vertexIndex,
            direction: e.direction,
            socket: e.socket,
        };

        connectionBeingCreated = {
            x1: mouseX,
            y1: mouseY,
            x2: mouseX,
            y2: mouseY,
        };
    }

    function handleSocketMouseup(event: CustomEvent<SocketEvent>) {
        let e = event.detail;

        // can't connect one node to the same node
        if (e.vertexIndex === connectionBeingCreatedFrom.index) return;

        // can't connect input to output
        if (e.direction === connectionBeingCreatedFrom.direction) return;

        let newConnection: Connection;

        // if the user started dragging from the input side, be sure to
        // connect the output to the input, not the input to the output
        if (connectionBeingCreatedFrom.direction.variant === "Input") {
            newConnection = {
                fromNode: e.vertexIndex,
                toNode: connectionBeingCreatedFrom.index,
                data: {
                    fromSocket: e.socket,
                    toSocket: connectionBeingCreatedFrom.socket,
                },
            };
        } else {
            newConnection = {
                fromNode: connectionBeingCreatedFrom.index,
                toNode: e.vertexIndex,
                data: {
                    fromSocket: connectionBeingCreatedFrom.socket,
                    toSocket: e.socket,
                },
            };
        }

        ipcSocket.commit(
            [
                {
                    variant: "ConnectNodes",
                    data: {
                        from: newConnection.fromNode,
                        to: newConnection.toNode,
                        data: newConnection.data,
                        graph: $activeGraph.graphIndex,
                    },
                },
            ],
            false,
        );
    }

    function connectionToPoints(connection: Connection): {
        x1: number;
        y1: number;
        x2: number;
        y2: number;
    } {
        const fromXY = $activeGraph.getNodeSocketXy(
            connection.fromNode,
            connection.data.fromSocket,
            { variant: "Output" },
        );
        const toXY = $activeGraph.getNodeSocketXy(
            connection.toNode,
            connection.data.toSocket,
            { variant: "Input" },
        );

        return {
            x1: fromXY.x,
            y1: fromXY.y,
            x2: toXY.x,
            y2: toXY.y,
        };
    }

    async function breadcrumbChangeGraph(
        event: CustomEvent<{ index: VertexIndex }>,
    ) {
        while (
            path.length > 1 &&
            path[path.length - 1].index !== event.detail.index
        ) {
            path.pop();
        }

        let graph = await graphManager.getGraph(event.detail.index);
        activeGraph.next(graph);

        path = path;
    }

    async function changeGraphTo(graphIndex: VertexIndex, nodeTitle: string) {
        let graph = await graphManager.getGraph(graphIndex);

        activeGraph.next(graph);
        path = [
            ...path,
            {
                name: nodeTitle,
                index: graphIndex,
            },
        ];
    }

    function changeGraph(e: CustomEvent<any>) {
        const graphIndex = e.detail.graphIndex;
        const nodeTitle = e.detail.nodeTitle;

        changeGraphTo(graphIndex, nodeTitle);
    }

    onMount(async () => {
        zoomer = panzoom(nodeContainer, {
            smoothScroll: false,
            maxZoom: 1,
            minZoom: 0.1,
        });
    });
</script>

<svelte:window
    on:mousedown={onWindowMousedown}
    on:mousemove={onMousemove}
    on:keydown={onKeydown}
    on:keyup={onKeyup}
    on:mouseup={onMouseup}
    on:paste={onPaste}
/>

<div
    class="editor"
    style="width: {width}px; height: {height}px"
    class:selecting={shiftHeld}
    bind:this={editor}
    on:mousedown={onEditorMousedown}
    on:contextmenu|preventDefault={onContextMenu}
>
    <div style="position: relative; height: 0px;" bind:this={nodeContainer}>
        <div style="position: absolute; height: 0px; z-index: -10">
            {#each keyedConnections as [key, connection] (key)}
                <ConnectionUI {...connectionToPoints(connection)} />
            {/each}
            {#if connectionBeingCreated}
                {#if connectionBeingCreatedFrom.direction.variant === "Input"}
                    <ConnectionUI
                        x1={connectionBeingCreated.x2}
                        y1={connectionBeingCreated.y2}
                        x2={connectionBeingCreated.x1}
                        y2={connectionBeingCreated.y1}
                    />
                {:else}
                    <ConnectionUI {...connectionBeingCreated} />
                {/if}
            {/if}
        </div>
        <div style="z-index: 10;">
            {#each keyedNodes as [key, node, index] (key)}
                <Node
                    graph={$activeGraph}
                    wrapper={node}
                    nodeIndex={index}
                    onMousedown={onNodeMousedown}
                    x={node.uiData.x}
                    y={node.uiData.y}
                    title={node.uiData.title || ""}
                    selected={node.uiData.selected || false}
                    on:socketMousedown={handleSocketMousedown}
                    on:socketMouseup={handleSocketMouseup}
                    on:changeGraph={changeGraph}
                />
            {/each}
        </div>
        {#if selection}
            <div
                style="z-index: 20; position: absolute; left: {selection.fromX}px; top: {selection.fromY}px;
                    width: {selection.toX -
                    selection.fromX}px; height: {selection.toY -
                    selection.fromY}px"
                class="selection-box"
            />
        {/if}
    </div>
    <div class="breadcrumb-container" style="width: {width - 16}px">
        <Breadcrumb on:click={breadcrumbChangeGraph} {path} />
    </div>
    {#if createNodeMenu.visible}
        <div
            style="position: absolute; left: {createNodeMenu.x}px; top: {createNodeMenu.y}px;"
        >
            <NodeCreationMenu on:selected={createNode} />
        </div>
    {/if}
</div>

<style>
    .editor {
        overflow: hidden;
        background-color: #fafafa;
    }

    .breadcrumb-container {
        position: absolute;
        padding: 8px;
        margin: 0;
        background-color: #ddd;
        z-index: 20;
    }

    .selecting {
        cursor: crosshair;
    }

    .selection-box {
        border: 2px solid blue;
        background: rgba(0, 0, 255, 0.2);
    }
</style>
