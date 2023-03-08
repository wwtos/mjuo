<script lang="ts">
    import Node from "./Node.svelte";
    import ConnectionUI from "./Connection.svelte";
    import { graphManager } from "./state";
    import { onMount } from "svelte";
    import type { NodeGraph } from "../node-engine/node_graph";
    import { NodeWrapper, NODE_WIDTH } from "../node-engine/node";
    import {
        SocketDirection,
        socketToKey,
        Connection,
        SocketType,
    } from "../node-engine/connection";
    import type { IPCSocket } from "../util/socket";
    import panzoom from "panzoom";
    import {
        transformMouse,
        transformMouseRelativeToEditor,
    } from "../util/mouse-transforms";
    import type { BehaviorSubject, Subscription } from "rxjs";
    import type { SocketEvent } from "./socket";
    import NodeCreationMenu from "./NodeCreationMenu.svelte";
    import Breadcrumb from "./Breadcrumb.svelte";
    import type { Index } from "../ddgg/gen_vec";
    import type { VertexIndex } from "../ddgg/graph";
    import { deselectAll } from "./editor-utils";

    export let width = 400;
    export let height = 400;

    export let ipcSocket: IPCSocket;

    export let activeGraph: BehaviorSubject<NodeGraph>;
    let previousSubscriptions: Array<Subscription> = [];

    // TODO: remove debugging VVV
    window["ipcSocket"] = ipcSocket;
    $: window["graph"] = $activeGraph;

    let nodeTypeToCreate: string;

    let zoomer;

    ipcSocket.requestGraph({ index: 0, generation: 0 });

    let editor: HTMLDivElement;
    let nodeContainer: HTMLDivElement;

    // node being actively dragged as well as the mouse offset
    let draggedNode: null | VertexIndex = null;
    let draggedNodeWasDragged = false;
    let draggedOffset: null | [number, number] = null;
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
        socket: SocketType;
    };
    let path = [
        {
            name: "root",
            index: { index: 0, generation: 0 },
        },
    ];

    let selectedNodes: VertexIndex[] = [];

    const onKeydown = (event) => {
        if (event.ctrlKey) {
            switch (event.key) {
                case "z":
                    if (event.shiftKey) {
                        ipcSocket.redo();
                    } else {
                        ipcSocket.undo();
                    }
                    break;
                case "y":
                    ipcSocket.redo();
                    break;
            }
        } else {
            switch (event.key) {
                case "Delete":
                    const selected = $activeGraph.nodeStore
                        .getValue()
                        .filter(([{ data: node }, _]) => node?.uiData.selected)
                        .map(([_, index]) => index);

                    for (let index of selected) {
                        ipcSocket.removeNode($activeGraph.graphIndex, index);
                    }
                    break;
            }
        }
    };

    const onMousemove = ({ clientX, clientY }) => {
        // convert window coordinates to editor coordinates
        let [mouseX, mouseY] = transformMouseRelativeToEditor(
            editor,
            zoomer,
            clientX,
            clientY
        );

        // if the mouse was moved and we are dragging a node, update that node's position
        if (draggedNode) {
            let node = $activeGraph.getNode(draggedNode);

            draggedNodeWasDragged = true;

            node.uiData = {
                ...node.uiData,
                x: mouseX - draggedOffset[0],
                y: mouseY - draggedOffset[1],
            };

            $activeGraph.updateNode(draggedNode);
            $activeGraph.update();
        } else if (connectionBeingCreated) {
            connectionBeingCreated.x2 = mouseX;
            connectionBeingCreated.y2 = mouseY;
        }
    };

    const onContextMenu = (event: MouseEvent) => {
        createNodeMenu.visible = true;
        createNodeMenu.x = event.clientX - 125;
        createNodeMenu.y = event.clientY;

        return false;
    };

    const onWindowMousedown = (event: MouseEvent) => {
        createNodeMenu.visible = false;
    };

    const onMouseup = () => {
        if (draggedNode && draggedNodeWasDragged) {
            $activeGraph.markNodeAsUpdated(draggedNode);
            $activeGraph.update();

            draggedNodeWasDragged = false;
        }

        $activeGraph.writeChangedNodesToServer();

        zoomer.resume();
        draggedNode = null;
        connectionBeingCreated = null;
    };

    function createNode(
        nodeType: CustomEvent<{
            value: string;
            clientX: number;
            clientY: number;
        }>
    ) {
        let boundingRect = editor.getBoundingClientRect();

        let relativeX = nodeType.detail.clientX - boundingRect.x;
        let relativeY = nodeType.detail.clientY - boundingRect.y;

        let [mouseX, mouseY] = transformMouse(zoomer, relativeX, relativeY);

        ipcSocket.createNode($activeGraph.graphIndex, nodeType.detail.value, {
            x: mouseX - NODE_WIDTH / 2,
            y: mouseY - 30,
        });

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

            draggedNode = index;

            let node = $activeGraph.getNode(draggedNode);
            draggedOffset = [mouseX - node.uiData.x, mouseY - node.uiData.y];

            let touchedNodes = deselectAll($activeGraph);

            node.uiData = {
                ...node.uiData,
                selected: true,
            };

            ipcSocket.updateNodesUi($activeGraph, [
                ...touchedNodes,
                draggedNode,
            ]);
        }
    }

    function handleSocketMousedown(event: CustomEvent<SocketEvent>) {
        let e = event.detail;

        let boundingRect = editor.getBoundingClientRect();

        let relativeX = e.event.clientX - boundingRect.x;
        let relativeY = e.event.clientY - boundingRect.y;

        let [mouseX, mouseY] = transformMouse(zoomer, relativeX, relativeY);

        if (e.direction === SocketDirection.Input) {
            // see if it's already connected, in which case we're disconnecting it
            let connection = $activeGraph.getNodeInputConnectionImmediate(
                e.vertexIndex,
                e.type
            );

            // check if we are already connected
            if (connection) {
                let fullConnection: Connection = {
                    fromNode: connection.fromNode,
                    toNode: e.vertexIndex,
                    data: {
                        fromSocketType: connection.fromSocketType,
                        toSocketType: connection.toSocketType,
                    },
                };

                ipcSocket.disconnectNode(
                    $activeGraph.graphIndex,
                    fullConnection
                );

                // add the connection line back for connecting to something else
                connectionBeingCreatedFrom = {
                    index: connection.fromNode,
                    direction: SocketDirection.Output,
                    socket: connection.fromSocketType,
                };

                const fromNode = $activeGraph.getNode(connection.fromNode);
                const fromNodeXY = $activeGraph.getNodeSocketXy(
                    connection.fromNode,
                    connection.fromSocketType,
                    SocketDirection.Output
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
            socket: e.type,
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
        if (connectionBeingCreatedFrom.direction === SocketDirection.Input) {
            newConnection = {
                fromNode: e.vertexIndex,
                toNode: connectionBeingCreatedFrom.index,
                data: {
                    fromSocketType: e.type,
                    toSocketType: connectionBeingCreatedFrom.socket,
                },
            };
        } else {
            newConnection = {
                fromNode: connectionBeingCreatedFrom.index,
                toNode: e.vertexIndex,
                data: {
                    fromSocketType: connectionBeingCreatedFrom.socket,
                    toSocketType: e.type,
                },
            };
        }

        ipcSocket.connectNode($activeGraph.graphIndex, newConnection);
    }

    function connectionToPoints(connection: Connection): {
        x1: number;
        y1: number;
        x2: number;
        y2: number;
    } {
        const fromXY = $activeGraph.getNodeSocketXy(
            connection.fromNode,
            connection.data.fromSocketType,
            SocketDirection.Output
        );
        const toXY = $activeGraph.getNodeSocketXy(
            connection.toNode,
            connection.data.toSocketType,
            SocketDirection.Input
        );

        return {
            x1: fromXY.x,
            y1: fromXY.y,
            x2: toXY.x,
            y2: toXY.y,
        };
    }

    async function breadcrumbChangeGraph(event: CustomEvent<{ index: Index }>) {
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

    async function changeGraphTo(graphIndex: Index, nodeTitle: string) {
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

    window["changeGraphTo"] = changeGraphTo;

    function changeGraph(e: CustomEvent<any>) {
        const graphIndex = e.detail.graphIndex;
        const nodeTitle = e.detail.nodeTitle;

        changeGraphTo(graphIndex, nodeTitle);
    }

    onMount(async () => {
        zoomer = panzoom(nodeContainer);
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
    on:contextmenu={onContextMenu}
>
    <div style="position: relative; height: 0px;" bind:this={nodeContainer}>
        <div style="position: absolute; height: 0px; z-index: -10">
            {#each keyedConnections as [key, connection] (key)}
                <ConnectionUI {...connectionToPoints(connection)} />
            {/each}
            {#if connectionBeingCreated}
                {#if connectionBeingCreatedFrom.direction === SocketDirection.Input}
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
                    nodes={$activeGraph}
                    wrapper={node}
                    nodeIndex={index}
                    onMousedown={handleNodeMousedown}
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
    select {
        background-color: white;
    }

    .new-node {
        height: 50px;
        position: absolute;
        background-color: lightgray;
        padding: 4px;
    }

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
