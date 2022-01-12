<script>
    const RADIUS = 12;

    export let x = 300;
    export let y = 300;
    export let type = "Stream";

    console.log(type);
</script>

{#if type === "Stream"}
    <circle cx={x} cy={y} r={RADIUS} class="socket" />
{:else if type === "Midi" }
    <rect x={x - RADIUS} y={y - RADIUS} width={RADIUS * 2} height={RADIUS * 2} class="midi" />
{:else if type === "Value" }
    <polygon points="{x - RADIUS},{y + RADIUS} {x},{y - RADIUS} {x + RADIUS},{y + RADIUS}" class="value" />
{/if}

<style>
.socket {
    fill: #96b38a;
    stroke: white;
}

.midi {
    fill: gold;
    stroke: white;
}

.value {
    fill: rgb(255, 166, 0);
    stroke: white;
}
</style>