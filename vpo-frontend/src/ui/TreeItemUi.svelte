<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { TreeItem } from "./tree-types";

    export let item: TreeItem;

    export let open = true;
    export let paddingLeft = 4;
    export let root = false;
    export let path = "";
    export let selected = false;

    const dispatch = createEventDispatcher();

    const toggle = () => {
        open = !open;
        // selected = true;
    };

    const pathClick = () => {
        dispatch("pathClick", {
            path: path + (item.children ? "/" : ""),
        });
    };
</script>

<div class="container">
    {#if item.children && item.children.length > 0}
        {#if !root}
            <div
                class="item"
                style="padding-left: {paddingLeft}px"
                on:click={toggle}
                on:dblclick={pathClick}
                class:selected
            >
                {#if open}
                    ▾
                {:else}
                    ▸
                {/if}
                {item.name}
            </div>
        {/if}
        <div class="children" class:open>
            {#each item.children as child (child.name)}
                <svelte:self
                    item={child}
                    paddingLeft={paddingLeft + 12}
                    path={path + "/" + child.name}
                    on:pathClick
                />
            {/each}
        </div>
    {:else}
        <div
            class="item"
            style="padding-left: {paddingLeft}px"
            on:click={toggle}
            on:dblclick={pathClick}
            class:selected
        >
            {item.name}
        </div>
    {/if}
</div>

<style>
    .container {
        width: 100%;
    }

    .item {
        padding: 4px;
        user-select: none;
        cursor: pointer;
    }

    .item:hover {
        background-color: #ddd;
    }

    .children {
        display: none;
    }

    .open {
        display: block;
    }

    .selected {
        background-color: rgb(110, 165, 248) !important;
        color: white;
    }
</style>
