<script lang="ts">
    import type { Writable } from "svelte/store";

    import type { IpcSocket } from "$lib/ipc/socket";
    import type { GraphManager } from "$lib/node-engine/graph_manager";
    import type { SocketRegistry } from "$lib/node-engine/socket_registry";
    import type { GlobalState } from "$lib/node-engine/global_state";
    import SplitView from "$lib/components/layout/SplitView.svelte";
    import { SplitDirection } from "$lib/components/layout/enums";
    import Container from "$lib/components/layout/Container.svelte";
    import UiElement from "./UiElement.svelte";
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import type { VertexIndex } from "$lib/ddgg/graph";
    import type { NodeWrapper } from "$lib/node-engine/node";

    export let socket: IpcSocket;
    export let graphManager: GraphManager;
    export let globalState: Writable<GlobalState>;

    export let width: number;
    export let height: number;

    let draggableWidth = 0;
    let windowHeight = 0;

    let uiNodes: Array<{
        index: VertexIndex;
        node: NodeWrapper;
        uiName: string;
    }>;

    $: graph = graphManager.getRootGraph();
    $: nodeStore = graph.nodeStore;

    $: uiNodes = $nodeStore
        .filter(([node, _]) => node.data.state.value !== null)
        .map(([node, index]) => ({
            index,
            node: node.data,
            uiName: node.data.properties.ui_name.data as string,
        }));

    function onDragStart(ev: DragEvent) {
        let nodeIndex = (ev.target as HTMLElement).dataset.nodeIndex as string;

        ev.dataTransfer?.setData("application/json", nodeIndex);
    }

    function onDragOver(ev: DragEvent) {
        ev.preventDefault();
        (ev.dataTransfer as DataTransfer).dropEffect = "copy";
    }

    function onDrop(ev: DragEvent) {
        const x = ev.offsetX;
        const y = ev.offsetY;

        const nodeIndex: VertexIndex = JSON.parse(
            ev.dataTransfer?.getData("application/json") || "{}"
        );

        const node = graph.getNode(nodeIndex);

        if (!node) return;

        const newElementInstance = {
            resourceId: "",
            properties: {},
            x,
            y,
        };

        if (!node.uiData.panelInstances) {
            node.uiData.panelInstances = {
                "0": [],
            };
        }

        node.uiData.panelInstances["0"].push(newElementInstance);

        graph.markNodeAsUpdated(nodeIndex);
        graph.writeChangedNodesToServer();
    }

    function onNewPosition(
        nodeIndex: VertexIndex,
        elementIndex: number,
        x: number,
        y: number
    ) {
        const node = graph.getNode(nodeIndex);

        if (!node?.uiData.panelInstances) return;

        node.uiData.panelInstances["0"][elementIndex].x = x;
        node.uiData.panelInstances["0"][elementIndex].y = y;

        console.log(nodeIndex, elementIndex, x, y);

        graph.markNodeAsUpdated(nodeIndex);
        graph.writeChangedNodesToServerUi();
    }

    $: console.log(uiNodes);
</script>

<SplitView
    {width}
    {height}
    bind:firstHeight={windowHeight}
    direction={SplitDirection.HORIZONTAL}
    initialSplitRatio={0.8}
>
    <div slot="first" style="border-bottom: 1px solid black">
        <SplitView
            {width}
            height={windowHeight}
            direction={SplitDirection.VERTICAL}
            bind:firstWidth={draggableWidth}
            initialSplitRatio={0.2}
        >
            <div
                slot="first"
                style={`min-width: ${draggableWidth}px; border-right: 1px solid black`}
                on:dragstart={onDragStart}
            >
                {#each uiNodes as { uiName, index }}
                    <div
                        class="ui-name"
                        draggable="true"
                        data-node-index={JSON.stringify(index)}
                    >
                        {uiName}
                    </div>
                {/each}
            </div>
            <div
                class="panel"
                slot="second"
                on:dragover={onDragOver}
                on:drop={onDrop}
                style={`min-width: ${
                    width - draggableWidth
                }px; min-height: ${height}px; position: relative`}
            >
                {#each uiNodes as { node, index, uiName } (index)}
                    {#if node.uiData["panelInstances"] && node.uiData.panelInstances["0"]}
                        {#each node.uiData.panelInstances["0"] as uiElement, elementIndex}
                            <UiElement
                                resourceId={uiElement.resourceId}
                                x={uiElement.x}
                                y={uiElement.y}
                                nodeType={node.nodeType}
                                state={node.state}
                                on:newposition={(e) =>
                                    onNewPosition(
                                        index,
                                        elementIndex,
                                        e.detail.x,
                                        e.detail.y
                                    )}
                                on:newstate={(e) =>
                                    socket.updateNodeState([[index, e.detail]])}
                                {uiName}
                            />
                        {/each}
                    {/if}
                {/each}
            </div>
        </SplitView>
    </div>
    <div slot="second">foobarbaz</div>
</SplitView>

<style>
    .ui-name {
        padding: 4px;
        background-color: #eee;
        user-select: none;
    }

    .ui-name:hover {
        background-color: #ddd;
    }

    .bordered {
        border: 1px solid black;
    }

    .panel {
        overflow: auto;
    }
</style>
