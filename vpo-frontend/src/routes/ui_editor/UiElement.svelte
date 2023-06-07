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
    export let properties: { [key: string]: string };
    export let globalState: Writable<GlobalState>;
    export let locked = false;

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

        if (!locked) {
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

    function updateStateCheckbox(e: MouseEvent) {
        updateState((e.target as HTMLInputElement).checked || false);
    }

    function updateStateClick(e: MouseEvent) {
        if (locked && state.value !== undefined) {
            updateState(!state.value);
        }
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
                <div
                    on:click={updateStateClick}
                    style={resourceJson.style || ""}
                >
                    {#if state.value === false}
                        {#each resourceJson.off.layer as layer, layerIndex}
                            <UiLayer {properties} {layer} {layerIndex} />
                        {/each}
                    {:else}
                        {#each resourceJson.on.layer as layer, layerIndex}
                            <UiLayer {properties} {layer} {layerIndex} />
                        {/each}
                    {/if}
                </div>
            {/if}
        </div>
    {:else}
        <span style="border: 1px solid black; padding: 2px">
            {uiName}
            {#if nodeType == "ToggleNode"}
                <input
                    type="checkbox"
                    checked={state.value}
                    on:click={updateStateCheckbox}
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
