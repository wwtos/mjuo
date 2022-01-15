<script lang="ts">
    import Node from "./Node.svelte";
    import { onMount } from 'svelte';
    import { writable } from 'svelte/store';

    export let width = 400;
    export let height = 400;

    $: changeDimensions(width, height);
    

    let editor: SVGElement;
    let mouseMoveStore = writable([0, 0]);
    let viewportStore = writable({
        left: 48,
        top: 0,
        width,
        height
    });

    function changeDimensions(width: number, height: number) {
        if (editor && width && height) {
            editor.setAttribute("viewBox", `0 0 ${width} ${height}`);
            editor.style.width = width + "px";
            editor.style.height = height + "px";

            viewportStore.set({
                left: 48,
                top: 0,
                width,
                height
            });
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
    <Node mouseStore={mouseMoveStore} viewportStore={viewportStore} />
</svg>

<style>
</style>