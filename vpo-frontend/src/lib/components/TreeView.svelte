<script lang="ts">
    import type { TreeItem } from "./tree-types";
    import TreeItemUi from "./TreeItemUi.svelte";

    export let items: TreeItem[] = [];
    export let width = 400;
    export let height = 400;
</script>

<div style="width: {width}px; height: {height}px">
    <TreeItemUi
        paddingLeft={-8}
        root={true}
        item={{ name: "", children: items }}
    />
</div>

<style>
    div {
        background-color: #eee;
        overflow: auto;
    }
</style>
