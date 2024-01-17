<script lang="ts">
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import { BehaviorSubject } from "rxjs";
    import type { PageData } from "./$types";
    import SideNavbar from "./node_editor/SideNavbar.svelte";
    import NodeEditor from "./node_editor/NodeEditor.svelte";
    import SettingsEditor from "./settings_editor/SettingsEditor.svelte";
    import FileEditor from "./file_editor/FileEditor.svelte";
    import UiEditor from "./ui_editor/UiEditor.svelte";

    export let data: PageData;

    let dimensions = data.windowDimensions;
    let section = "nodeEditor";

    $: width = $dimensions.width;
    $: height = $dimensions.height;

    let activeGraph: BehaviorSubject<NodeGraph> = new BehaviorSubject(
        data.graphManager.getRootGraph(),
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
        />
    {:else if section === "uiEditor"}
        <UiEditor
            graphManager={data.graphManager}
            globalState={data.globalEngineState}
            resources={data.globalResources}
            socket={data.socket}
            width={width - 48}
            {height}
        />
    {:else if section === "fileEditor"}
        <FileEditor globalState={data.globalEngineState} socket={data.socket} />
    {:else if section === "settings"}
        <SettingsEditor
            socket={data.socket}
            globalState={data.globalEngineState}
            {activeGraph}
        />
    {/if}
</div>
