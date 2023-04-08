<script>
    export let x1 = 0;
    export let y1 = 0;
    export let x2 = 100;
    export let y2 = 100;

    $: minX = Math.min(x1, x2);
    $: minY = Math.min(y1, y2);
    $: maxX = Math.max(x1, x2);
    $: maxY = Math.max(y1, y2);

    const curvature = 0.4;

    $: hx1 = x1 + Math.abs(x2 - x1) * curvature;
    $: hx2 = x2 - Math.abs(x2 - x1) * curvature;
</script>

<svg
    viewBox="{minX - 50} {minY - 50} {maxX - minX + 100} {maxY - minY + 100}"
    style="width: {maxX - minX + 100}px; height: {maxY -
        minY +
        100}px; transform: translate({minX - 50}px, {minY - 50}px)"
>
    <path d="M {x1} {y1} C {hx1} {y1} {hx2} {y2} {x2} {y2}" />
</svg>

<style>
    svg {
        margin: 0px;
        position: absolute;
    }

    path {
        stroke-width: 5px;
        stroke: steelblue;
        fill: none;
    }
</style>
