<script>
export let onchange = function() {};
export let value;
export let step = 1;

let firstTime = true;

$: {
    console.log("Number property got value", value);

    if(!firstTime && value !== undefined) {
        onchange(value);
    }

    if (firstTime) {
        firstTime = false;
    }
}
</script>

<input type="number" bind:value={value} step={step} />

<style>
input {
    width: 100%;
    margin: 0;
}
</style>