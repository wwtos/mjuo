<script lang="ts">
    import type { VertexIndex } from "$lib/ddgg/graph";
    import type { GlobalState } from "$lib/node-engine/global_state";
    import type { NodeWrapper } from "$lib/node-engine/node";
    import { createEventDispatcher, onMount } from "svelte";
    import type { Writable } from "svelte/store";
    import { parse } from "toml";
    import UiLayer from "./UiLayer.svelte";

    export let resourceId: string;
    export let uiName: string;
    export let x: number = 0;
    export let y: number = 0;
    export let selected = false;
    export let nodeType: string;
    export let state: NodeWrapper["state"];
    export let globalState: Writable<GlobalState>;
    export let choosable = false;

    let resourceJson: any = {};
    $: if (resourceId.length > 0) {
        resourceJson =
            $globalState.resources.ui[
                resourceId.substring(resourceId.indexOf(":"))
            ];
    }

    const dispatchEvent = createEventDispatcher();

    let anchorX = 0;
    let anchorY = 0;

    let dragging = false;

    function onMousedown(e: MouseEvent) {
        anchorX = e.clientX - x;
        anchorY = e.clientY - y;

        if (!choosable) {
            dragging = true;
        } else {
            dispatchEvent("skinselected", resourceId);
        }
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
    style={`left: ${x}px; top: ${y}px`}
    on:mousedown={onMousedown}
    class="container"
    class:selected
>
    {#if "type" in resourceJson}
        <div
            style={`width: ${resourceJson.width}px; height: ${resourceJson.height}px; position: relative`}
        >
            {#if resourceJson.type === "off/on"}
                {#if state.value === false}
                    {#each resourceJson.off.layer as layer}
                        <UiLayer {layer} />
                    {/each}
                {:else}
                    {#each resourceJson.on.layer as layer}
                        <UiLayer {layer} />
                    {/each}
                {/if}
            {/if}
        </div>
    {:else}
        <span style="border: 1px solid black; padding: 2px">
            {uiName}
            {#if nodeType == "ToggleNode"}
                <input
                    type="checkbox"
                    checked={state.value}
                    on:click={(e) => updateState(e.target?.checked || false)}
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

    .selected {
        outline: blue solid 3px;
    }
</style>
