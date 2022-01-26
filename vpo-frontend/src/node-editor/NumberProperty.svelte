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

<input type="number" style="width: 100%" bind:value={value} />
<style>
input {
    height: 26px;
    border: none;
    margin: 0;
    outline: none;
    border-radius: 0;
    box-shadow: none;
    resize: none;
}

input:focus-visible {
    border: 1px solid blue;
    border-radius: 0;
}
</style>