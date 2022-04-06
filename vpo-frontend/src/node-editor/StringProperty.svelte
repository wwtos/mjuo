<script>
export let onchange = function() {};
export let value;

let firstTime = true;

$: {
    if(!firstTime && value !== undefined) {
        onchange(value);
    }

    if (firstTime) {
        firstTime = false;
    }
}
</script>

<input type="text" bind:value={value} />

<style>
input {
    width: 100%;
    margin: 0;
}
</style>