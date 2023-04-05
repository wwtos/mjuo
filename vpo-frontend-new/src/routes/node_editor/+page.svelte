<script lang="ts">
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import { BehaviorSubject } from "rxjs";
    import type { PageData } from "./$types";
    import NodeEditor from "./NodeEditor.svelte";

    export let data: PageData;

    let dimensions = data.windowDimensions;

    $: width = $dimensions.width;
    $: height = $dimensions.height;

    let activeGraph: BehaviorSubject<NodeGraph> = new BehaviorSubject(
        data.graphManager.getRootGraph()
    );
</script>

<NodeEditor
    width={width - 48}
    {height}
    {activeGraph}
    graphManager={data.graphManager}
    ipcSocket={data.socket}
    socketRegistry={data.socketRegistry}
/>
