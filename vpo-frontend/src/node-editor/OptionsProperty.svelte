<script lang="ts">
export let onchange: Function = function() {};
export let value;
export let states: string[]; // {"id": "value"}

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
{#each states as state}
    <option value={state}>{state}</option>
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
