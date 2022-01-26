<script>
export let onchange = function() {};
export let value;
export let states; // {"id": "value"}

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

<select bind:value={value}>
{#each Object.entries(states) as [i, state]}
    <option value={i}>{state}</option>
{/each}
</select>

<style>
select {
    width: 100%;
    height: 26px;
    border: none;
    margin: 0;
    outline: none;
    border-radius: 0;
    box-shadow: none;
    resize: none;
    padding: 0;
    background-color: white;
}

select:focus-visible {
    border: 1px solid blue;
    border-radius: 0;
}
</style>