<script lang="ts">
    import Node from "./Node.svelte";
    import Connection from "./Connection.svelte";
    import { onMount } from 'svelte';
    import { Graph, PossibleNode } from '../node-engine/graph';
    import { NodeIndex, NodeWrapper } from "../node-engine/node";
    import { MidiSocketType, SocketDirection, socketToKey, SocketType, StreamSocketType, Connection as ConnectionObj } from "../node-engine/connection";
    import { EnumInstance } from "../util/enum";
    import { IPCSocket } from "../util/socket";
    import panzoom from "panzoom";
    import { transformMouse } from "../util/mouse-transforms";
    
    export let width = 400;
    export let height = 400;

    export let ipcSocket: IPCSocket;

    export let nodes: Graph = new Graph();

    // TODO: remove debugging VVV
    window["ipcSocket"] = ipcSocket;
    window["graph"] = nodes;


    let nodeTypeToCreate: string;

    let zoomer;

    ipcSocket.send({
        "action": "graph/get"
    });

    ipcSocket.onMessage(message => {
        console.log("received", message);

        if (message.action === "graph/updateGraph") {
            nodes.applyJson(message.payload);
        }
    });

    let editor: HTMLDivElement;
    let nodeContainer: HTMLDivElement;
    let draggedNode: (null | NodeIndex) = null;
    let draggedOffset: (null | [number, number]) = null;

    let connectionBeingCreated: (null | {x1: number, y1: number, x2: number, y2: number}) = null;
    let connectionBeingCreatedFrom: {
        index: NodeIndex,
        direction: SocketDirection,
        socket: EnumInstance
    };

    let selectedNodes: NodeIndex[] = [];
    let nodeSocketPositionMapping = {};

    onMount(async () => {
        window.addEventListener("mousemove", ({clientX, clientY}) => {
            let boundingRect = editor.getBoundingClientRect();

            let relativeX = clientX - boundingRect.x;
            let relativeY = clientY - boundingRect.y;

            let [mouseX, mouseY] = transformMouse(zoomer, relativeX, relativeY);

            if (draggedNode) {
                let node = nodes.getNode(draggedNode);

                node.uiData.x = mouseX - draggedOffset[0];
                node.uiData.y = mouseY - draggedOffset[1];

                nodes.update();
            } else if (connectionBeingCreated) {
                connectionBeingCreated.x2 = mouseX;
                connectionBeingCreated.y2 = mouseY;
            }            
        });

        window.addEventListener("mouseup", function() {
            if (draggedNode) {
                ipcSocket.send({
                    "action": "graph/updateNodes",
                    "payload": [
                        JSON.parse(JSON.stringify(nodes.getNode(draggedNode)))
                    ]
                });
            }

            zoomer.resume();
            draggedNode = null;
            connectionBeingCreated = null;
        });

        zoomer = panzoom(nodeContainer);
    });

    function createNode () {
        ipcSocket.createNode(nodeTypeToCreate);
    }

    let keyedNodes;
    let keyedConnections;

    nodes.subscribeToKeyedNodes().subscribe(newKeyedNodes => {
        keyedNodes = newKeyedNodes;
    });

    nodes.subscribeToKeyedConnections().subscribe(newKeyedConnections => {
        keyedConnections = newKeyedConnections;
    });

    function handleNodeMousedown (index: NodeIndex, event: MouseEvent) {
        if (event.button === 0) {
            let boundingRect = editor.getBoundingClientRect();

            let relativeX = event.clientX - boundingRect.x;
            let relativeY = event.clientY - boundingRect.y;

            let [mouseX, mouseY] = transformMouse(zoomer, relativeX, relativeY);
            
            zoomer.pause();

            draggedNode = index;

            let node = nodes.getNode(draggedNode);
            draggedOffset = [mouseX - node.uiData.x, mouseY - node.uiData.y];

            //node.uiData.selected = true;
        }
    }

    function handleSocketMousedown(event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection, index: NodeIndex) {
        let boundingRect = editor.getBoundingClientRect();

        let relativeX = event.clientX - boundingRect.x;
        let relativeY = event.clientY - boundingRect.y;

        let [mouseX, mouseY] = transformMouse(zoomer, relativeX, relativeY);

        if (direction === SocketDirection.Input) {
            // see if it's already connected, in which case we're disconnecting it
            const disconnectingFrom = nodes.getNode(index);

            let connection = disconnectingFrom.connectedInputs.find(inputSocket => {
                return socketToKey(inputSocket.toSocketType, direction) === socketToKey(socket, direction);
            });


            if (connection) {
                let fullConnection = new ConnectionObj(
                    connection.fromSocketType,
                    connection.fromNode,
                    connection.toSocketType,
                    index
                );

                ipcSocket.disconnectNode(fullConnection);

                // add the connection line back for connecting to something else
                connectionBeingCreatedFrom = {
                    index: connection.fromNode,
                    direction: SocketDirection.Output,
                    socket: connection.fromSocketType
                };

                const fromNode = nodes.getNode(connection.fromNode);
                const fromNodeKey = connection.fromNode.toKey();
                const fromNodeSockets = nodeSocketPositionMapping[fromNodeKey];
                const fromSocket = fromNodeSockets[socketToKey(connection.fromSocketType, SocketDirection.Output)];

                connectionBeingCreated = {
                    x1: fromSocket.x + fromNode.uiData.x,
                    y1: fromSocket.y + fromNode.uiData.y,
                    x2: mouseX,
                    y2: mouseY
                }

                return;
            }
        }

        connectionBeingCreatedFrom = {
            index,
            direction,
            socket
        };
        
        connectionBeingCreated = {
            x1: mouseX,
            y1: mouseY,
            x2: mouseX,
            y2: mouseY
        };
    }

    function handleSocketMouseup(event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection, index: NodeIndex) {
        // can't connect one node to the same node
        if (index.index === connectionBeingCreatedFrom.index.index) return;

        // can't connect input to output
        if (direction === connectionBeingCreatedFrom.direction) return;

        let newConnection;

        // if the user started dragging from the input side, be sure to
        // connect the output to the input, not the input to the output
        if (connectionBeingCreatedFrom.direction === SocketDirection.Input) {
            newConnection = new ConnectionObj(
                socket,
                index,
                connectionBeingCreatedFrom.socket,
                connectionBeingCreatedFrom.index
            );
        } else {
            newConnection = new ConnectionObj(
                connectionBeingCreatedFrom.socket,
                connectionBeingCreatedFrom.index,
                socket,
                index
            );
        }

        ipcSocket.send(JSON.parse(JSON.stringify({
            "action": "graph/connectNode",
            "payload": newConnection
        })));
    }

    function handleExportSocketPositionMapping(socketPositionMapping: ({ [key: string]: DOMRect }), key: string, nodeLocation: DOMRect) {
        // calculate relative values for all the node sockets
        nodeSocketPositionMapping[key] = socketPositionMapping;

        Object.keys(nodeSocketPositionMapping[key]).forEach(function(innerKey, index) {
            var pos = nodeSocketPositionMapping[key][innerKey];

            if (!pos.width) return;

            var mappedPos = transformMouse(
                zoomer,
                pos.x + (pos.width / 2) - nodeLocation.x,
                pos.y + (pos.height / 2) - nodeLocation.y
            );

            nodeSocketPositionMapping[key][innerKey] = {
                x: mappedPos[0],
                y: mappedPos[1]
            };
        });
    }

    function connectionToPoints(connection: any): {x1: number, y1: number, x2: number, y2: number} {
        const fromNode = nodes.getNode(connection.fromNode);
        const fromNodeKey = connection.fromNode.toKey();
        const fromNodeSockets = nodeSocketPositionMapping[fromNodeKey];

        const toNode = nodes.getNode(connection.toNode);
        const toNodeKey = connection.toNode.toKey();
        const toNodeSockets = nodeSocketPositionMapping[toNodeKey];

        if (!fromNodeSockets) return {
            x1: 0, y1: 0, x2: 0, y2: 0
        };

        const fromSocket = fromNodeSockets[socketToKey(connection.fromSocketType, SocketDirection.Output)];
        const toSocket = toNodeSockets[socketToKey(connection.toSocketType, SocketDirection.Input)];

        return {
            x1: fromSocket.x + fromNode.uiData.x,
            y1: fromSocket.y + fromNode.uiData.y,
            x2: toSocket.x + toNode.uiData.x,
            y2: toSocket.y + toNode.uiData.y
        };
    }

    setTimeout(() => {
        keyedConnections = keyedConnections;
    }, 500);
</script>

<div class="editor" style="width: {width}px; height: {height}px" bind:this={editor}>
    <div style="position: relative; height: 0px;" bind:this={nodeContainer}>
        <div style="position: absolute; height: 0px; z-index: -10">
            {#each keyedConnections as [key, connection] (key) }
                <Connection {...connectionToPoints(connection)} />
            {/each}
            {#if connectionBeingCreated}
                {#if connectionBeingCreatedFrom.direction === SocketDirection.Input}
                    <Connection x1={connectionBeingCreated.x2} y1={connectionBeingCreated.y2} x2={connectionBeingCreated.x1} y2={connectionBeingCreated.y1} />
                {:else}
                    <Connection {...connectionBeingCreated} />
                {/if}
                
            {/if}
        </div>
        <div style="z-index: 10">
            {#each keyedNodes as [key, node] (key) }
                <Node wrapper={node} onMousedown={handleNodeMousedown} onSocketMousedown={handleSocketMousedown} onSocketMouseup={handleSocketMouseup} exportSocketPositionMapping={handleExportSocketPositionMapping} />
            {/each}
        </div>
    </div>

    <div class="new-node" style="width: {width - 9}px">
        New node type:
        <select bind:value={nodeTypeToCreate}>
            <option value="GainGraphNode">Gain graph node</option>
        </select>
        <button on:click={createNode}>Create!</button>
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
}
</style>