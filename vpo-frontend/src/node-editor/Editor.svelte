<script lang="ts">
    import Node from "./Node.svelte";
    import { onMount } from 'svelte';
    import { Graph, PossibleNode } from '../node-engine/graph';
    import { NodeIndex, NodeWrapper } from "../node-engine/node";
    import { MidiSocketType, SocketType, StreamSocketType } from "../node-engine/connection";
    import { EnumInstance } from "../util/enum";
    import { IPCSocket } from "../util/socket";
    
    export let width = 400;
    export let height = 400;

    export let ipcSocket: IPCSocket;

    let nodeTypeToCreate: string;

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

    let viewportLeft: number = 0;
    let viewportTop: number = 0;
    let viewportWidth: number = width;
    let viewportHeight: number = height;

    // whenever the editor is given a new size, perform the appropriate calculations
    // to readjust the various sub components and variables
    function changeDimensions(newWidth: number, newHeight: number) {
        if (newWidth && newHeight) {
            width = newWidth;
            height = newHeight;

            viewportWidth = width;
            viewportHeight = height;
        }
    }

    onMount(async () => {
        changeDimensions(width, height);

        window.addEventListener("mousemove", ({clientX, clientY}) => {
            let boundingRect = editor.getBoundingClientRect();

            let relativeX = clientX - boundingRect.x;
            let relativeY = clientY - boundingRect.y;
        });
    });

    function createNode () {
        console.log(nodeTypeToCreate);

        ipcSocket.send({
            "action": "graph/newNode",
            "payload": nodeTypeToCreate
        });
    }
</script>

<div class="editor" style="width: {width}px; height: {height}px" bind:this={editor}>
    <!-- TODO: yes, I'm lazy, if things start breaking maybe fix this rect -->
    <div class="new-node" style="width: {width - 9}px">
        New node type:
        <select bind:value={nodeTypeToCreate}>
            <option value="GainGraphNode">Gain graph node</option>
        </select>
        <button on:click={createNode}>Create!</button>
    </div>
    
    {#each nodes.getKeyedNodes() as [key, node] (key) }
        <Node wrapper={node} />
    {/each}
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