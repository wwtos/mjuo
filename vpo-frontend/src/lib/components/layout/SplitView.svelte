<!-- thanks to vscode for inspiring this window design! -->
<script lang="ts">
    import { onMount } from "svelte";
    import { SplitDirection } from "./enums";

    export let direction: SplitDirection;
    export let width: number;
    export let height: number;

    export let canResize = true;
    export let hasFixedWidth = false;
    export let fixedWidth = 0;
    export let initialSplitRatio = 0.5;

    export let firstWidth: number = 0;
    export let firstHeight: number = 0;
    export let secondWidth = 0;
    export let secondHeight = 0;

    $: if (hasFixedWidth) {
        canResize = false;
    }

    let container: HTMLElement;

    let currentlyResizingDivider = false;

    function dividerMousedown() {
        if (canResize) {
            currentlyResizingDivider = true;
        }
    }

    if (!hasFixedWidth) {
        switch (direction) {
            case SplitDirection.VERTICAL:
                firstWidth = Math.floor(width * initialSplitRatio);

                firstHeight = height;
                break;
            case SplitDirection.HORIZONTAL:
                firstHeight = Math.floor(height * initialSplitRatio);

                firstWidth = width;
                break;
        }
    } else {
        firstWidth = fixedWidth;
    }

    onMount(async () => {
        window.addEventListener("mousemove", (e) => {
            let { clientX, clientY } = e;

            if (currentlyResizingDivider) {
                e.preventDefault(); // stop the text from being selected during drag

                const containerPos = container.getBoundingClientRect();

                if (direction === SplitDirection.VERTICAL) {
                    firstWidth = clientX - containerPos.left;
                } else if (direction === SplitDirection.HORIZONTAL) {
                    firstHeight = clientY - containerPos.top;
                }
            }
        });

        window.addEventListener("mouseup", function () {
            currentlyResizingDivider = false;
        });
    });

    $: switch (direction) {
        case SplitDirection.VERTICAL:
            secondWidth = width - firstWidth;
            secondHeight = height;

            break;
        case SplitDirection.HORIZONTAL:
            secondWidth = width;
            secondHeight = height - firstHeight;

            break;
    }
</script>

{#if direction === SplitDirection.VERTICAL}
    <div
        class="container vertical-split"
        style="width: {width}px; height: {height}px"
        bind:this={container}
    >
        {#if canResize}
            <div class="divider-parent">
                <div
                    class="divider divider-vertical"
                    class:dragging={currentlyResizingDivider}
                    style="left: {firstWidth - 2}px; height: {height}px"
                    on:mousedown={dividerMousedown}
                />
            </div>
        {/if}
        <div style="width: {firstWidth}px; height: {height}px">
            <slot name="first" />
        </div>
        <div style="width: {width - firstWidth}px; height: {height}px">
            <slot name="second" />
        </div>
    </div>
{:else if direction === SplitDirection.HORIZONTAL}
    <div
        class="container horizontal-split"
        style="width: {width}px; height: {height}px"
        bind:this={container}
    >
        {#if canResize}
            <div class="divider-parent">
                <div
                    class="divider divider-horizontal"
                    class:dragging={currentlyResizingDivider}
                    style="top: {firstHeight - 2}px; width: {width}px"
                    on:mousedown={dividerMousedown}
                />
            </div>
        {/if}
        <div style="width: {width}px; height: {firstHeight}px">
            <slot name="first" />
        </div>
        <div style="width: {width}px; height: {height - firstHeight}px">
            <slot name="second" />
        </div>
    </div>
{/if}

<style>
    .container.horizontal-split {
        display: flex;
        flex-direction: column;
    }

    .container.vertical-split {
        display: flex;
        flex-direction: row;
    }

    .divider-parent {
        position: relative;
    }

    .divider {
        position: absolute;
        z-index: 10;
        transition: background-color 0.2s;
    }

    .divider.divider-horizontal {
        left: 0px;
        height: 4px;
    }

    .divider.divider-vertical {
        top: 0px;
        width: 4px;
    }

    .divider.divider-vertical:hover,
    .divider.dragging.divider-vertical {
        background-color: lightskyblue;
        cursor: ew-resize;
    }

    .divider.divider-horizontal:hover,
    .divider.dragging.divider.divider-horizontal {
        background-color: lightskyblue;
        cursor: ns-resize;
    }
</style>
