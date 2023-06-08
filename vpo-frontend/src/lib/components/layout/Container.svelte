<script lang="ts">
    export let width: number;
    export let height: number;
</script>

<div style={`width: ${width}px; height: ${height}px`}>
    <slot />
</div>
