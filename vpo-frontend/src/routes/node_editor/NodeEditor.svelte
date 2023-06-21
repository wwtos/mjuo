<script lang="ts">
    import Node from "./Node.svelte";
    import ConnectionUI from "./Connection.svelte";
    import { onMount } from "svelte";
    import panzoom, { type PanZoom } from "panzoom";
    import type { BehaviorSubject, Subscription } from "rxjs";
    import type { SocketEvent } from "./socket";
    import NodeCreationMenu from "./NodeCreationMenu.svelte";
    import Breadcrumb from "./Breadcrumb.svelte";
    import { deselectAll } from "./editor-utils";
    import type { IpcSocket } from "$lib/ipc/socket";
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import type { VertexIndex } from "$lib/ddgg/graph";
    import {
        transformMouse,
        transformMouseRelativeToEditor,
    } from "$lib/util/mouse-transforms";
    import { NODE_WIDTH, type NodeWrapper } from "$lib/node-engine/node";
    import type { GraphManager } from "$lib/node-engine/graph_manager";
    import type { SocketRegistry } from "$lib/node-engine/socket_registry";
    import type {
        Socket,
        SocketDirection,
        Connection,
    } from "$lib/node-engine/connection";
    import type { Action } from "$lib/node-engine/state";

    export let width = 400;
    export let height = 400;

    export let ipcSocket: IpcSocket;
    export let activeGraph: BehaviorSubject<NodeGraph>;
    export let graphManager: GraphManager;
    export let socketRegistry: SocketRegistry;

    let previousSubscriptions: Array<Subscription> = [];

    let nodeTypeToCreate: string;

    let zoomer: PanZoom;

    ipcSocket.requestGraph({ index: 0, generation: 0 });

    let editor: HTMLDivElement;
    let nodeContainer: HTMLDivElement;

    // node being actively dragged as well as the mouse offset
    let draggedState: {
        node: VertexIndex;
        offset: [number, number];
    } | null = null;

    let draggedNodeWasDragged = false;
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
    let path = [
        {
            name: "root",
            index: { index: 0, generation: 0 },
        },
    ];

    let controlHeld = false;

    function onKeydown(event: KeyboardEvent) {
        controlHeld = event.ctrlKey;

        if (!event.ctrlKey) {
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
    }

    function onMousemove({ clientX, clientY }: MouseEvent) {
        // convert window coordinates to editor coordinates
        let [mouseX, mouseY] = transformMouseRelativeToEditor(
            editor,
            zoomer,
            clientX,
            clientY
        );

        // if the mouse was moved and we are dragging a node, update that node's position
        if (draggedState) {
            let node = $activeGraph.getNode(draggedState.node) as NodeWrapper;

            draggedNodeWasDragged = true;

            node.uiData = {
                ...node.uiData,
                x: mouseX - draggedState.offset[0],
                y: mouseY - draggedState.offset[1],
            };

            $activeGraph.updateNode(draggedState.node);
            $activeGraph.update();
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

    function onMouseup() {
        if (draggedState && draggedNodeWasDragged) {
            $activeGraph.markNodeAsUpdated(draggedState.node, ["uiData"]);
            $activeGraph.update();

            draggedNodeWasDragged = false;
        }

        $activeGraph.writeChangedNodesToServer();

        zoomer.resume();
        draggedState = null;
        connectionBeingCreated = null;
    }

    function createNode(
        ev: CustomEvent<{
            value: string;
            clientX: number;
            clientY: number;
        }>
    ) {
        let boundingRect = editor.getBoundingClientRect();

        let relativeX = ev.detail.clientX - boundingRect.x;
        let relativeY = ev.detail.clientY - boundingRect.y;

        let [mouseX, mouseY] = transformMouse(zoomer, relativeX, relativeY);

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

    let keyedNodes: [string, NodeWrapper, VertexIndex][];
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
                }
            ),
        ];
    }

    function handleNodeMousedown(index: VertexIndex, event: MouseEvent) {
        if (event.button === 0) {
            let boundingRect = editor.getBoundingClientRect();

            let relativeX = event.clientX - boundingRect.x;
            let relativeY = event.clientY - boundingRect.y;

            let [mouseX, mouseY] = transformMouse(zoomer, relativeX, relativeY);

            zoomer.pause();

            let node = $activeGraph.getNode(index) as NodeWrapper;

            draggedState = {
                node: index,
                offset: [mouseX - node.uiData.x, mouseY - node.uiData.y],
            };

            if (!controlHeld) {
                let touchedNodes = deselectAll($activeGraph);

                for (let touched of touchedNodes) {
                    $activeGraph.markNodeAsUpdated(touched, ["uiData"]);
                }
            }

            node = $activeGraph.getNode(index) as NodeWrapper;

            node.uiData = {
                ...node.uiData,
                selected: true,
            };

            $activeGraph.markNodeAsUpdated(index, ["uiData"]);
            $activeGraph.writeChangedNodesToServer();
        }
    }

    function handleSocketMousedown(event: CustomEvent<SocketEvent>) {
        let e = event.detail;

        let boundingRect = editor.getBoundingClientRect();

        let relativeX = e.event.clientX - boundingRect.x;
        let relativeY = e.event.clientY - boundingRect.y;

        let [mouseX, mouseY] = transformMouse(zoomer, relativeX, relativeY);

        if (e.direction.variant === "Input") {
            // see if it's already connected, in which case we're disconnecting it
            let connection = $activeGraph.getNodeInputConnection(
                e.vertexIndex,
                e.socket
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
                    false
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
                    { variant: "Output" }
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
        if (e.vertexIndex.index === connectionBeingCreatedFrom.index.index)
            return;

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
            false
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
            { variant: "Output" }
        );
        const toXY = $activeGraph.getNodeSocketXy(
            connection.toNode,
            connection.data.toSocket,
            { variant: "Input" }
        );

        return {
            x1: fromXY.x,
            y1: fromXY.y,
            x2: toXY.x,
            y2: toXY.y,
        };
    }

    async function breadcrumbChangeGraph(
        event: CustomEvent<{ index: VertexIndex }>
    ) {
        while (
            path.length > 1 &&
            path[path.length - 1].index.index !== event.detail.index.index
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
    on:mouseup={onMouseup}
/>

<div
    class="editor"
    style="width: {width}px; height: {height}px"
    bind:this={editor}
    on:keydown={onKeydown}
    on:keyup={onKeyup}
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
        <div style="z-index: 10">
            {#each keyedNodes as [key, node, index] (key)}
                <Node
                    graph={$activeGraph}
                    wrapper={node}
                    nodeIndex={index}
                    onMousedown={handleNodeMousedown}
                    registry={socketRegistry}
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
</style>
