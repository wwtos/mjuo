<script lang="ts">
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import { BehaviorSubject } from "rxjs";
    import type { PageData } from "./$types";
    import NodeEditor from "./NodeEditor.svelte";

    export let data: PageData;

    let dimensions = data.windowDimensions;

    let width = $dimensions.width;
    let height = $dimensions.height;

    let activeGraph: BehaviorSubject<NodeGraph> = new BehaviorSubject(
        data.graphManager.getRootGraph()
    );
</script>

<NodeEditor
    {width}
    {height}
    {activeGraph}
    graphManager={data.graphManager}
    ipcSocket={data.socket}
    socketRegistry={data.socketRegistry}
/>
