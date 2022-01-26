<!-- thanks to vscode for inspiring this window design! -->
<script type="ts">
    import { onMount } from 'svelte';
    import {SplitDirection} from "./enums";
    
    export let direction: SplitDirection;
    export let width: number;
    export let height: number;

    export let firstPanel: any;
    export let secondPanel: any;
    export let firstState: object = {};
    export let secondState: object = {};

    export let canResize = true;
    export let hasFixedWidth = false;
    export let fixedWidth = 0;

    $: if (hasFixedWidth) {
        canResize = false;
    }

    let firstWidth: number, firstHeight: number;
    let container: HTMLElement | undefined;

    let currentlyResizingDivider = false;

    function dividerMousedown () {
        if (canResize) {
            currentlyResizingDivider = true;
        }
    }

    if (!hasFixedWidth) {
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
    } else {
        firstWidth = fixedWidth;
    }

    onMount(async () => {
        window.addEventListener("mousemove", e => {
            let {clientX, clientY} = e;

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

        window.addEventListener("mouseup", function() {
            currentlyResizingDivider = false;
        });
    });
</script>

{#if direction === SplitDirection.VERTICAL}
<div class="container vertical-split" style="width: {width}px; height: {height}px" bind:this={container}>
    {#if canResize}
        <div class="divider-parent">
            <div class="divider divider-vertical" class:dragging={currentlyResizingDivider} style="left: {firstWidth - 2}px; height: {height}px" on:mousedown={dividerMousedown}></div>
        </div>
    {/if}
    <svelte:component this={firstPanel} width={firstWidth} height={height} {...firstState} />
    <svelte:component this={secondPanel} width={width - firstWidth} height={height} {...secondState} />
</div>
{:else if direction === SplitDirection.HORIZONTAL}
<div class="container horizontal-split" style="width: {width}px; height: {height}px" bind:this={container}>
    {#if canResize}
        <div class="divider-parent">
            <div class="divider divider-horizontal" class:dragging={currentlyResizingDivider} style="top: {firstHeight - 2}px; width: {width}px" on:mousedown={dividerMousedown}></div>
        </div>
    {/if}
    <svelte:component this={firstPanel} width={width} height={firstHeight} {...firstState} />
    <svelte:component this={secondPanel} width={width} height={height - firstHeight} {...secondState} />
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

.divider.divider-vertical {
    top: 0px;
    width: 4px;
}

.divider:hover, .divider.dragging {
    background-color: lightskyblue;
    cursor: ew-resize;
}
</style>
