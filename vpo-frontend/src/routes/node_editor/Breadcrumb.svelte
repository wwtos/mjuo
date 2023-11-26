<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let path = [
        {
            name: "root",
            index: "0.0",
        },
    ];

    const dispatch = createEventDispatcher();

    const onClick = (index: string) => {
        dispatch("click", {
            index,
        });
    };
</script>

{#each path as resource (resource.index)}
    <a href="#" on:click={() => onClick(resource.index)}>{resource.name}</a> /&nbsp;
{/each}
