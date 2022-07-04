<script lang="ts">
    import Node from "./Node.svelte";
    import Connection from "./Connection.svelte";
    import { socketRegistry } from "./state"
    import { onMount } from 'svelte';
    import { Graph, PossibleNode } from '../node-engine/graph';
    import { NodeIndex, NodeWrapper } from "../node-engine/node";
    import { MidiSocketType, SocketDirection, socketToKey, Connection as ConnectionObj, SocketType } from "../node-engine/connection";
    import { IPCSocket } from "../util/socket";
    import panzoom from "panzoom";
    import { transformMouse, transformMouseRelativeToEditor } from "../util/mouse-transforms";
    import { variants } from "../node-engine/variants";
    import { i18nStore } from '../i18n.js';
    import { get } from "svelte/store";
    import { MemberType } from "safety-match";
    
    export let width = 400;
    export let height = 400;

    export let ipcSocket: IPCSocket;

    export let nodes: Graph;

    // TODO: remove debugging VVV
    window["ipcSocket"] = ipcSocket;
    window["graph"] = nodes;


    let nodeTypeToCreate: string;

    let zoomer;

    ipcSocket.send({
        "action": "graph/get"
    });

    ipcSocket.onMessage(([message]) => {
        console.log("received", message);

        if (message.action === "graph/updateGraph") {
            nodes.applyJson(message.payload);
        } else if (message.action === "registry/updateRegistry") {
            $socketRegistry.applyJson(message.payload);

            console.log($socketRegistry);
        }
    });

    let editor: HTMLDivElement;
    let nodeContainer: HTMLDivElement;

    // node being actively dragged as well as the mouse offset
    let draggedNode: (null | NodeIndex) = null;
    let draggedOffset: (null | [number, number]) = null;

    // the points of the connection being created
    let connectionBeingCreated: (null | {x1: number, y1: number, x2: number, y2: number}) = null;
    let connectionBeingCreatedFrom: {
        index: NodeIndex,
        direction: SocketDirection,
        socket: MemberType<typeof SocketType>
    };

    let selectedNodes: NodeIndex[] = [];

    // map a node socket to its xy coords in the editor
    let nodeSocketPositionMapping = {};

    onMount(async () => {
        window.addEventListener("mousemove", ({clientX, clientY}) => {
            // convert window coordinates to editor coordinates
            let [mouseX, mouseY] = transformMouseRelativeToEditor(editor, zoomer, clientX, clientY);

            // if the mouse was moved and we are dragging a node, update that node's position
            if (draggedNode) {
                let node = nodes.getNode(draggedNode);

                node.uiData.next({
                    ...node.uiData.getValue(),
                    x: mouseX - draggedOffset[0],
                    y: mouseY - draggedOffset[1]

                });

                nodes.update();
            } else if (connectionBeingCreated) {
                connectionBeingCreated.x2 = mouseX;
                connectionBeingCreated.y2 = mouseY;
            }            
        });

        window.addEventListener("mouseup", function() {
            if (draggedNode) {
                nodes.markNodeAsUpdated(draggedNode);
                nodes.update();
            }

            nodes.writeChangedNodesToServer();

            zoomer.resume();
            draggedNode = null;
            connectionBeingCreated = null;
        });

        zoomer = panzoom(nodeContainer);
    });

    function createNode () {
        ipcSocket.createNode(nodeTypeToCreate, {
            x: 0,
            y: 0
        });
    }

    let keyedNodes;
    let keyedConnections;

    nodes.keyedNodeStore.subscribe(newKeyedNodes => {
        keyedNodes = newKeyedNodes;
    });

    nodes.keyedConnectionStore.subscribe(newKeyedConnections => {
        keyedConnections = newKeyedConnections;
    });

    function deselectAll () {
        const currentNodes = nodes.nodeStore.getValue();

        for (var i = 0; i < currentNodes.length; i++) {
            if (currentNodes[i] && currentNodes[i].uiData.getValue().selected) {
                nodes.markNodeAsUpdated(currentNodes[i].index);

                currentNodes[i].uiData.next({
                    ...currentNodes[i].uiData.getValue(),
                    selected: false
                });
            }
        }
    }

    function handleNodeMousedown (index: NodeIndex, event: MouseEvent) {
        if (event.button === 0) {
            let boundingRect = editor.getBoundingClientRect();

            let relativeX = event.clientX - boundingRect.x;
            let relativeY = event.clientY - boundingRect.y;

            let [mouseX, mouseY] = transformMouse(zoomer, relativeX, relativeY);
            
            zoomer.pause();

            draggedNode = index;

            let node = nodes.getNode(draggedNode);
            draggedOffset = [mouseX - node.uiData.getValue().x, mouseY - node.uiData.getValue().y];

            deselectAll();

            node.uiData.next({
                ...node.uiData.getValue(),
                selected: true
            });
        }
    }

    function handleSocketMousedown(event: MouseEvent, socket: MemberType<typeof SocketType>, direction: SocketDirection, index: NodeIndex) {
        let boundingRect = editor.getBoundingClientRect();

        let relativeX = event.clientX - boundingRect.x;
        let relativeY = event.clientY - boundingRect.y;

        let [mouseX, mouseY] = transformMouse(zoomer, relativeX, relativeY);

        if (direction === SocketDirection.Input) {
            // see if it's already connected, in which case we're disconnecting it
            const disconnectingFrom = nodes.getNode(index);

            let connection = disconnectingFrom.connectedInputs.getValue().find(inputSocket => {
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
                const fromNodeXY = fromNode.getSocketXYCurrent(connection.fromSocketType, SocketDirection.Output);

                connectionBeingCreated = {
                    x1: fromNodeXY[0],
                    y1: fromNodeXY[1],
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

    function handleSocketMouseup(event: MouseEvent, socket: MemberType<typeof SocketType>, direction: SocketDirection, index: NodeIndex) {
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

    function handleExportSocketPositionMapping(socketPositionMapping: ({ [key: string]: [number, number] }), key: string) {
        nodeSocketPositionMapping[key] = socketPositionMapping;
    }

    function connectionToPoints(connection: any): {x1: number, y1: number, x2: number, y2: number} {
        const fromNode = nodes.getNode(connection.fromNode);
        const toNode = nodes.getNode(connection.toNode);

        const fromXY = fromNode.getSocketXYCurrent(connection.fromSocketType, SocketDirection.Output);
        const toXY = toNode.getSocketXYCurrent(connection.toSocketType, SocketDirection.Input);


        return {
            x1: fromXY[0],
            y1: fromXY[1],
            x2: toXY[0],
            y2: toXY[1]
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
                <Node {nodes} wrapper={node} onMousedown={handleNodeMousedown} onSocketMousedown={handleSocketMousedown} onSocketMouseup={handleSocketMouseup} />
            {/each}
        </div>
    </div>

    <div class="new-node" style="width: {width - 9}px">
        {$i18nStore.t('editor.newNodeType')}
        <select bind:value={nodeTypeToCreate} on:mousedown={e => e.stopPropagation()}>
            {#each variants as {name, internal} }
                <option value="{internal}">{name}</option>
            {/each}
        </select>
        <button on:click={createNode}>{$i18nStore.t('editor.create')}</button>
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