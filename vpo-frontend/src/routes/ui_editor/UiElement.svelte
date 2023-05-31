<script lang="ts">
    import { FILE_PREFIX } from "$lib/constants";
    import type { VertexIndex } from "$lib/ddgg/graph";
    import type { NodeWrapper } from "$lib/node-engine/node";
    import { createEventDispatcher, onMount } from "svelte";

    export let resourceId: string;
    export let uiName: string;
    export let x: number;
    export let y: number;
    export let key: number | string;
    export let nodeIndex: VertexIndex;
    export let nodeType: string;
    export let state: NodeWrapper["state"];

    const dispatchEvent = createEventDispatcher();

    let anchorX = 0;
    let anchorY = 0;

    let dragging = false;

    onMount(async () => {
        console.log(FILE_PREFIX + resourceId.replace(":", "/"));
    });

    function onMousedown(e: MouseEvent) {
        anchorX = e.clientX - x;
        anchorY = e.clientY - y;

        dragging = true;
    }

    function onMousemove(e: MouseEvent) {
        if (dragging) {
            x = e.clientX - anchorX;
            y = e.clientY - anchorY;
        }
    }

    function onMouseup() {
        if (dragging) {
            dragging = false;

            dispatchEvent("newposition", {
                x: x,
                y: y,
            });
        }
    }

    function updateState(state: any) {
        dispatchEvent("newstate", state);
    }
</script>

<svelte:window on:mousemove={onMousemove} on:mouseup={onMouseup} />

<div
    class="container"
    style={`left: ${x}px; top: ${y}px`}
    on:mousedown={onMousedown}
>
    {#if resourceId.length > 0}{:else}
        <span style="border: 1px solid black; padding: 2px">
            {uiName}
            {#if nodeType == "ToggleNode"}
                <input
                    type="checkbox"
                    checked={state.value}
                    on:click={(e) => updateState(e.target.checked || false)}
                    on:mousedown|stopPropagation
                    on:mouseup|stopPropagation
                />
            {/if}
        </span>
    {/if}
</div>

<style>
    .container {
        position: absolute;
        display: inline-block;
        z-index: auto;
        user-select: none;
    }
</style>
