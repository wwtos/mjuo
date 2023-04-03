<script>
    export let onchange = function () {};
    export let value;
    export let step = 1;

    let firstTime = true;

    $: {
        if (!firstTime && value !== undefined) {
            onchange(value);
        }

        if (firstTime) {
            firstTime = false;
        }
    }
</script>

<input type="number" bind:value {step} />

<style>
    input {
        width: 100%;
        margin: 0;
    }
</style>
