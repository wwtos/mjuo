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
    background-color: white;
    padding: 0;
    margin: 0;
}
</style>
