<script lang="ts">
    export let value: string;
</script>

<input type="text" bind:value on:change />

<style>
    input {
        width: 100%;
        margin: 0;
    }
</style>
