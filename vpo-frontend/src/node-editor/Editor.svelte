<script lang="ts">
    import Node from "./Node.svelte";
    import { onMount } from 'svelte';
    import { Graph, PossibleNode } from '../node-engine/graph';
    import { NodeIndex, NodeWrapper } from "../node-engine/node";
    import { MidiSocketType, SocketType, StreamSocketType } from "../node-engine/connection";
    import { EnumInstance } from "../util/enum";
    import { IPCSocket } from "../util/socket";
    import panzoom from "panzoom";
    import { transformMouse } from "../util/mouse-transforms";
    
    export let width = 400;
    export let height = 400;

    export let ipcSocket: IPCSocket;

    let nodeTypeToCreate: string;

    let zoomer;

    $: changeDimensions(width, height);

    export let nodes: Graph = new Graph();

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

    let selectedNodes: NodeIndex[] = [];

    // whenever the editor is given a new size, perform the appropriate calculations
    // to readjust the various sub components and variables
    function changeDimensions(newWidth: number, newHeight: number) {
        if (newWidth && newHeight) {
            width = newWidth;
            height = newHeight;
        }
    }

    onMount(async () => {
        changeDimensions(width, height);

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
            }
            
        });

        window.addEventListener("mouseup", function() {
            if (draggedNode) {
                ipcSocket.send({
                    "action": "graph/updateNodes",
                    "payload": [
                        nodes.getNode(draggedNode).toJSON()
                    ]
                });
            }

            zoomer.resume();
            draggedNode = null;
        });

        zoomer = panzoom(nodeContainer);

        console.log(zoomer);
    });

    function createNode () {
        console.log(nodeTypeToCreate);

        ipcSocket.send({
            "action": "graph/newNode",
            "payload": nodeTypeToCreate
        });
    }

    let keyedNodes = nodes.getKeyedNodes();

    nodes.subscribeToKeyedNodes().subscribe(newKeyedNodes => {
        keyedNodes = newKeyedNodes;
    });

    function handleMousedown (index: NodeIndex, event: MouseEvent) {
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
</script>

<div class="editor" style="width: {width}px; height: {height}px" bind:this={editor}>
    <div style="position: relative; height: 0px;" bind:this={nodeContainer}>
        {#each keyedNodes as [key, node] (key) }
            <Node wrapper={node} onMousedown={handleMousedown} />
        {/each}
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