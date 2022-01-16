<!-- thanks to vscode for inspiring this window design! -->
<script type="ts">
    import { onMount } from 'svelte';
    import {SplitDirection} from "./enums";
    
    export let direction: SplitDirection;
    export let width: number;
    export let height: number;

    let firstWidth, firstHeight;
    let container;

    let currentlyResizingDivider = false;

    function dividerMousedown (e) {
        currentlyResizingDivider = true;
    }

    switch (direction) {
        case SplitDirection.VERTICAL:
            firstWidth = Math.floor(width / 2);

            firstHeight = height;
        break;
        case SplitDirection.HORIZONTAL:
            firstHeight = Math.floor(height / 2);

            firstWidth = width;
        break;
    }

    onMount(async () => {
        window.addEventListener("mousemove", ({clientX, clientY}) => {
            if (currentlyResizingDivider) {
                const containerPos = container.getBoundingClientRect();

                if (direction === SplitDirection.VERTICAL) {
                    firstWidth = clientX - containerPos.left;
                }
            }
        });

        window.addEventListener("mouseup", function() {
            currentlyResizingDivider = false;
        })
    });
</script>

{#if direction === SplitDirection.VERTICAL}
<div class="container vertical-split" style="width: {width}px; height: {height}px" bind:this={container}>
    <slot name="first" firstWidth={firstWidth} firstHeight={height}></slot>
    <slot name="second" secondWidth={width - firstWidth} secondHeight={height}></slot>
    <div class="divider divider-vertical" class:dragging={currentlyResizingDivider} style="left: {firstWidth - 2}px; height: {height}px" on:mousedown={dividerMousedown}></div>
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

.divider {
    position: absolute;
    z-index: 10;
    transition: background-color 0.2s;
}

.divider.divider-vertical {
    top: 0px;
    width: 4px;
}

.divider:hover, .divider.dragging {
    background-color: lightskyblue;
    cursor: ew-resize;
}
</style>
