<script lang="ts">
    import { type NodeVariant, variants } from "$lib/node-engine/variants";
    import { Localized } from "@nubolab-ffwd/svelte-fluent";
    import { createEventDispatcher } from "svelte";
    import { localize } from "@nubolab-ffwd/svelte-fluent";

    const dispatch = createEventDispatcher();

    export let menuWidth = 250;
    let openCategory: null | string = null;

    let categories: { [key: string]: NodeVariant[] } = variants.reduce(
        (acc, val) => {
            return {
                ...acc,
                [val.category]: [],
            };
        },
        {}
    );

    for (let node of variants) {
        categories[node.category].push(node);
    }

    for (let category in categories) {
        categories[category].sort((a, b) =>
            $localize("node." + a.internal).localeCompare(
                $localize("node." + b.internal)
            )
        );
    }

    let categoryNames = Object.keys(categories);
    categoryNames.sort();

    const selectCategory = (category: string) => {
        openCategory = category;
    };

    const valueSelected = (value: string, event: MouseEvent) => {
        dispatch("selected", {
            value,
            clientX: event.clientX,
            clientY: event.clientY,
        });
    };
</script>

<div class="menu" style="width: {menuWidth}px">
    {#each categoryNames as category (category)}
        <div
            class="category item"
            on:mouseenter={() => selectCategory(category)}
            on:mousedown|stopPropagation
        >
            {#if openCategory === category}
                <div
                    style="position: absolute; left: {menuWidth}px; width: {menuWidth}px;"
                >
                    <div class="menu" style="width: {menuWidth}px">
                        {#each categories[category] as nodeType (nodeType.internal)}
                            <div
                                class="item"
                                on:click={(event) =>
                                    valueSelected(nodeType.internal, event)}
                            >
                                {$localize("node." + nodeType.internal)}
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}
            {category}
            <span style="float: right; margin-left: 4px">▸</span>
        </div>
    {/each}
</div>

<style>
    .menu {
        border: solid 1px black;
        background-color: white;
        font-size: 1.2rem;
    }

    span,
    div {
        user-select: none;
    }

    .item {
        padding: 4px;
    }

    .item:hover {
        background-color: #ddd;
    }
</style>
