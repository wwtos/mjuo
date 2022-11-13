<script lang="ts">
    import SplitView from "../layout/SplitView.svelte";
    import TreeView from "../ui/TreeView.svelte";
    import { SplitDirection } from "../layout/enums";
    import FileView from "./FileView.svelte";
    import type { TreeItem } from "../ui/tree-types";
    import { globalState } from "../node-editor/state";

    export let width: number = 400;
    export let height: number = 400;

    let files: TreeItem[] = [];
    $: files = Object.keys($globalState.resources).map((namespace) => ({
        name: namespace,
        children: $globalState.resources[namespace].reduce((acc, val) => {
            const pathParts = val.split("/");

            let traversal = acc;

            for (let i = 0; i < pathParts.length; i++) {
                let itemIndex = traversal.findIndex(
                    (item) => item.name === pathParts[i]
                );

                if (itemIndex === -1) {
                    itemIndex = traversal.length;

                    traversal.push({
                        name: pathParts[i],
                        children: i < pathParts.length - 1 ? [] : undefined,
                    });
                }

                traversal = traversal[itemIndex].children;
            }

            return acc;
        }, []),
    }));
</script>

<SplitView
    direction={SplitDirection.VERTICAL}
    {width}
    {height}
    firstPanel={TreeView}
    firstState={{ items: files }}
    secondPanel={FileView}
/>
