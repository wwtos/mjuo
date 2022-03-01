<script lang="ts">
    import Node from "./Node.svelte";
    import { onMount } from 'svelte';
    import { Graph } from '../node-engine/graph';
    
    export let width = 400;
    export let height = 400;

    $: changeDimensions(width, height);

    export let nodes: Graph;

    let editor: SVGElement;

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

    function backgroundMousedown () {
        console.log("here");
    }
</script>

<svg viewBox="{viewportLeft} {viewportTop} {viewportWidth} {viewportHeight}">
    <!-- TODO: yes, I'm lazy, if things start breaking maybe fix this rect -->
    <rect x="-10000000" y="-10000000" width="20000000" height="20000000" opacity="0" on:mousedown={backgroundMousedown} />
    
    {#each nodes.getKeyedNodes() as [key, node] (key) }
        <Node wrapper={node} />
    {/each}
</svg>

<style>
    svg {
        border: 1px solid black;
    }
</style>