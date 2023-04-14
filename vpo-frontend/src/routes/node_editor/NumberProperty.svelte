<script lang="ts">
    export let value: number;
    export let step = 1;
</script>

<input type="number" {value} {step} on:change />

<style>
    input {
        width: 100%;
        margin: 0;
    }
</style>
