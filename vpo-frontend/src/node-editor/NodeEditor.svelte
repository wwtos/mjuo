<script lang="ts">
    import Node from "./Node.svelte";
    import ConnectionUI from "./Connection.svelte";
    import { graphManager, socketRegistry } from "./state";
    import { onMount } from "svelte";
    import { NodeIndex } from "../node-engine/node_index";
    import { NodeGraph, PossibleNode } from "../node-engine/node_graph";
    import { NodeWrapper } from "../node-engine/node";
    import {
        MidiSocketType,
        SocketDirection,
        socketToKey,
        Connection,
        SocketType,
    } from "../node-engine/connection";
    import { IPCSocket } from "../util/socket";
    import panzoom from "panzoom";
    import { transformMouse, transformMouseRelativeToEditor } from "../util/mouse-transforms";
    import { variants } from "../node-engine/variants";
    import { i18nStore } from "../i18n.js";
    import { get } from "svelte/store";
    import { MemberType } from "safety-match";
    import { BehaviorSubject, Subject, Subscription } from "rxjs";
    import type { SocketEvent } from "./socket";

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

    ipcSocket.requestGraph(0);

    let editor: HTMLDivElement;
    let nodeContainer: HTMLDivElement;

    // node being actively dragged as well as the mouse offset
    let draggedNode: null | NodeIndex = null;
    let draggedNodeWasDragged = false;
    let draggedOffset: null | [number, number] = null;

    // the points of the connection being created
    let connectionBeingCreated: null | {
        x1: number;
        y1: number;
        x2: number;
        y2: number;
    } = null;
    let connectionBeingCreatedFrom: {
        index: NodeIndex;
        direction: SocketDirection;
        socket: SocketType;
    };

    let selectedNodes: NodeIndex[] = [];

    onMount(async () => {
        editor.addEventListener("keydown", (event) => {
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
                    case "s":
                        ipcSocket.save();
                        break;
                }
            } else {
                switch (event.key) {
                    case "Delete":
                        const selected = $activeGraph.nodeStore
                            .getValue()
                            .filter((node) => node?.ui_data.selected)
                            .map((node) => node.index);

                        for (let index of selected) {
                            ipcSocket.removeNode($activeGraph.graphIndex, index);
                        }
                        break;
                }
            }
        });

        window.addEventListener("mousemove", ({ clientX, clientY }) => {
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

                node.ui_data = {
                    ...node.ui_data,
                    x: mouseX - draggedOffset[0],
                    y: mouseY - draggedOffset[1],
                };

                $activeGraph.updateNode(node.index);
                $activeGraph.update();
            } else if (connectionBeingCreated) {
                connectionBeingCreated.x2 = mouseX;
                connectionBeingCreated.y2 = mouseY;
            }
        });

        window.addEventListener("mouseup", function () {
            if (draggedNode && draggedNodeWasDragged) {
                $activeGraph.markNodeAsUpdated(draggedNode);
                $activeGraph.update();

                draggedNodeWasDragged = false;
            }

            $activeGraph.writeChangedNodesToServer();

            zoomer.resume();
            draggedNode = null;
            connectionBeingCreated = null;
        });

        zoomer = panzoom(nodeContainer);
    });

    function createNode(e: MouseEvent) {
        e.stopPropagation();

        ipcSocket.createNode($activeGraph.graphIndex, nodeTypeToCreate, {
            x: 0,
            y: 0,
        });
    }

    let keyedNodes: [string, NodeWrapper][];
    let keyedConnections: [string, Connection][];

    $: {
        if (previousSubscriptions.length > 0) {
            previousSubscriptions.forEach((sub) => sub.unsubscribe());
        }

        previousSubscriptions = [
            $activeGraph.keyedNodeStore.subscribe((newKeyedNodes) => {
                keyedNodes = newKeyedNodes;
            }),
            $activeGraph.keyedConnectionStore.subscribe((newKeyedConnections) => {
                keyedConnections = newKeyedConnections;
            }),
        ];
    }

    function deselectAll(): NodeIndex[] {
        const currentNodes = $activeGraph.nodeStore.getValue();

        let touchedNodes: NodeIndex[] = [];

        for (var i = 0; i < currentNodes.length; i++) {
            if (currentNodes[i] && currentNodes[i].ui_data.selected) {
                touchedNodes.push(currentNodes[i].index);

                currentNodes[i].ui_data = {
                    ...currentNodes[i].ui_data,
                    selected: false,
                };

                $activeGraph.updateNode(currentNodes[i].index);
            }
        }

        return touchedNodes;
    }

    function handleNodeMousedown(index: NodeIndex, event: MouseEvent) {
        if (event.button === 0) {
            let boundingRect = editor.getBoundingClientRect();

            let relativeX = event.clientX - boundingRect.x;
            let relativeY = event.clientY - boundingRect.y;

            let [mouseX, mouseY] = transformMouse(zoomer, relativeX, relativeY);

            zoomer.pause();

            draggedNode = index;

            let node = $activeGraph.getNode(draggedNode);
            draggedOffset = [mouseX - node.ui_data.x, mouseY - node.ui_data.y];

            let touchedNodes = deselectAll();

            node.ui_data = {
                ...node.ui_data,
                selected: true,
            };

            ipcSocket.updateNodesUi($activeGraph.graphIndex, [
                ...touchedNodes.map((index) => $activeGraph.getNode(index)),
                node,
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
            const disconnectingFrom = $activeGraph.getNode(e.nodeIndex);

            let connection = disconnectingFrom.connected_inputs.find((inputSocket) => {
                return (
                    socketToKey(inputSocket.to_socket_type, e.direction) ===
                    socketToKey(e.type, e.direction)
                );
            });

            // check if we are already connected
            if (connection) {
                let fullConnection: Connection = {
                    from_socket_type: connection.from_socket_type,
                    from_node: connection.from_node,
                    to_socket_type: connection.to_socket_type,
                    to_node: e.nodeIndex,
                };

                ipcSocket.disconnectNode($activeGraph.graphIndex, fullConnection);

                // add the connection line back for connecting to something else
                connectionBeingCreatedFrom = {
                    index: connection.from_node,
                    direction: SocketDirection.Output,
                    socket: connection.from_socket_type,
                };

                const fromNode = $activeGraph.getNode(connection.from_node);
                const fromNodeXY = $activeGraph.getNodeSocketXY(
                    fromNode.index,
                    connection.from_socket_type,
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
            index: e.nodeIndex,
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
        if (e.nodeIndex.index === connectionBeingCreatedFrom.index.index) return;

        // can't connect input to output
        if (e.direction === connectionBeingCreatedFrom.direction) return;

        let newConnection: Connection;

        // if the user started dragging from the input side, be sure to
        // connect the output to the input, not the input to the output
        if (connectionBeingCreatedFrom.direction === SocketDirection.Input) {
            newConnection = {
                from_socket_type: e.type,
                from_node: e.nodeIndex,
                to_socket_type: connectionBeingCreatedFrom.socket,
                to_node: connectionBeingCreatedFrom.index,
            };
        } else {
            newConnection = {
                from_socket_type: connectionBeingCreatedFrom.socket,
                from_node: connectionBeingCreatedFrom.index,
                to_socket_type: e.type,
                to_node: e.nodeIndex,
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
        const fromXY = $activeGraph.getNodeSocketXY(
            connection.from_node,
            connection.from_socket_type,
            SocketDirection.Output
        );
        const toXY = $activeGraph.getNodeSocketXY(
            connection.to_node,
            connection.to_socket_type,
            SocketDirection.Input
        );

        return {
            x1: fromXY.x,
            y1: fromXY.y,
            x2: toXY.x,
            y2: toXY.y,
        };
    }

    async function changeGraphTo(graphIndex: number) {
        let graph = await graphManager.getGraph(graphIndex);

        activeGraph.next(graph);
    }

    window["changeGraphTo"] = changeGraphTo;

    function changeGraph(e: CustomEvent<any>) {
        const graphIndex = e.detail.graphIndex;

        changeGraphTo(graphIndex);
    }
</script>

<div class="editor" style="width: {width}px; height: {height}px" bind:this={editor}>
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
            {#each keyedNodes as [key, node] (key)}
                <Node
                    nodes={$activeGraph}
                    wrapper={node}
                    onMousedown={handleNodeMousedown}
                    on:socketMousedown={handleSocketMousedown}
                    on:socketMouseup={handleSocketMouseup}
                    on:changeGraph={changeGraph}
                />
            {/each}
        </div>
    </div>

    <div class="new-node" style="width: {width - 9}px">
        {$i18nStore.t("editor.newNodeType")}
        <select bind:value={nodeTypeToCreate} on:mousedown={(e) => e.stopPropagation()}>
            {#each variants as { name, internal }}
                <option value={internal}>{name}</option>
            {/each}
        </select>
        <button on:click={createNode}>{$i18nStore.t("editor.create")}</button>
    </div>
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
        border: 1px solid black;
        overflow: hidden;
        background-color: #fafafa;
    }
</style>
