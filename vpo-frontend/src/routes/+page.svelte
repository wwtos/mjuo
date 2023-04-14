<script lang="ts">
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import { BehaviorSubject } from "rxjs";
    import type { PageData } from "./$types";
    import SideNavbar from "./node_editor/SideNavbar.svelte";
    import NodeEditor from "./node_editor/NodeEditor.svelte";
    import SettingsEditor from "./settings_editor/SettingsEditor.svelte";

    export let data: PageData;

    let dimensions = data.windowDimensions;
    let section = "nodeEditor";

    $: width = $dimensions.width;
    $: height = $dimensions.height;

    let activeGraph: BehaviorSubject<NodeGraph> = new BehaviorSubject(
        data.graphManager.getRootGraph()
    );
</script>

<div style="display: flex">
    <SideNavbar
        on:click={(newSection) => (section = newSection.detail)}
        {section}
    />
    {#if section === "nodeEditor"}
        <NodeEditor
            width={width - 48}
            {height}
            {activeGraph}
            graphManager={data.graphManager}
            ipcSocket={data.socket}
            socketRegistry={data.socketRegistry}
        />
    {:else if section === "settings"}
        <SettingsEditor socket={data.socket} />
    {/if}
</div>
