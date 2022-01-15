<script lang="ts">
    import Node from "./Node.svelte";
    import { windowDimensions } from "../util/window-size";
    import { onMount } from 'svelte';
    import { writable } from 'svelte/store';

    export let width = 400;
    export let height = 400;

    $: changeDimensions(width, height);
    

    let editor;
    let mouseMoveStore = writable([0, 0]);

    function changeDimensions(width, height) {
        if (editor && width && height) {
            editor.setAttribute("viewBox", `0 0 ${width} ${height}`);
            editor.style.width = width + "px";
            editor.style.height = height + "px";
        }
    }

    onMount(async () => {
        changeDimensions(width, height);

        window.addEventListener("mousemove", ({clientX, clientY}) => {
            let boundingRect = editor.getBoundingClientRect();

            let relativeX = clientX - boundingRect.x;
            let relativeY = clientY - boundingRect.y;

            mouseMoveStore.set([relativeX, relativeY]);
        });
    });
</script>

<svg bind:this={editor} viewBox="0 0 220 100">
    <Node mouseStore={mouseMoveStore} />
</svg>

<style>
</style>