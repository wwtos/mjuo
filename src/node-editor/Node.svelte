<script>
    import Socket from "./Socket.svelte";

    const BACKGROUND_COLOR = "#c8a2c8";
    const ROUNDNESS = 7;
    const PADDING = 10;
    const PADDING_TOP = PADDING + 7;
    const TEXT_PADDING = 30;
    const SOCKET_LIST_START = 55;
    const TEXT_SIZE = 14;
    const SOCKET_VERTICAL_SPACING = TEXT_SIZE + 5;
    
    export let title = "Test title";
    export let properties = [
        ["Audio in", 
        {
            "type": "Stream",
            "content": [{
                "type": "Audio"
            }]
        }, "INPUT"],
        ["Audio out", 
        {
            "type": "Value",
            "content": [{
                "type": "Audio"
            }]
        }, "OUTPUT"]
    ];
    export let width = 200;
    export let x = 100;
    export let y = 100;
    export let mouseStore;

    let computedHeight = SOCKET_LIST_START + SOCKET_VERTICAL_SPACING * (properties.length - 1) + TEXT_SIZE + PADDING;

    let dragging = false;

    function clicked () {
        dragging = true;
    }

    function released () {
        dragging = false;
    }

    mouseStore.subscribe(([mouseX, mouseY]) => {
        if (dragging) {
            x = mouseX - 5;
            y = mouseY - 5;
        }
    });
</script>

<g transform="translate({x}, {y})">
<rect width="{width}" height="{computedHeight}" rx="{ROUNDNESS}" class="background" on:mousedown={clicked} on:mouseup={released} />
<text x={PADDING} y={PADDING_TOP} class="title" on:mousedown={clicked} on:mouseup={released}>{title}</text>

{#each properties as property, i (property[0])}
    {#if property[2] === "INPUT"}
        <text x={TEXT_PADDING} y={SOCKET_LIST_START + SOCKET_VERTICAL_SPACING * i} on:mousedown={clicked} on:mouseup={released}>{property[0]}</text>
        <Socket x="0" y={SOCKET_LIST_START + SOCKET_VERTICAL_SPACING * i} type={property[1].type} />
    {:else}
        <text x={width - TEXT_PADDING} y={SOCKET_LIST_START + SOCKET_VERTICAL_SPACING * i} class="right-align" on:mousedown={clicked} on:mouseup={released}>{property[0]}</text>
        <Socket x={width} y={SOCKET_LIST_START + SOCKET_VERTICAL_SPACING * i} type={property[1].type} />
    {/if}
{/each}
</g>

<style>
.background {
    fill: rgba(110, 136, 255, 0.8);
    stroke: #4e58bf;
    stroke-width: 2px;
}

text {
    text-anchor: start;
    dominant-baseline: central;
    font-size: 14px;
    font-family: sans-serif;
    fill: white;
    user-select: none;
}

.center-align {
    text-anchor: middle;
}

.right-align {
    text-anchor: end;
}

.title {
    font-size: 18px;
}
</style>