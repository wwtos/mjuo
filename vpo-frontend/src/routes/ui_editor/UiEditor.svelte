<script lang="ts">
    import type { Writable } from "svelte/store";

    import type { IpcSocket } from "$lib/ipc/socket";
    import type { GraphManager } from "$lib/node-engine/graph_manager";
    import type { GlobalState } from "$lib/node-engine/global_state";
    import SplitView from "$lib/components/layout/SplitView.svelte";
    import { SplitDirection } from "$lib/components/layout/enums";
    import UiElement from "./UiElement.svelte";
    import type { VertexIndex } from "$lib/ddgg/graph";
    import type { NodeWrapper } from "$lib/node-engine/node";

    export let socket: IpcSocket;
    export let graphManager: GraphManager;
    export let globalState: Writable<GlobalState>;

    export let width: number;
    export let height: number;

    let draggableWidth = 0;
    let viewportHeight = 0;

    let locked = true;
    let textareaContent = "";

    let currentlySelected: {
        nodeIndex: VertexIndex;
        instance: string;
        elementIndex: number;
    } | null = null;

    let uiNodes: Array<{
        index: VertexIndex;
        node: NodeWrapper;
        uiName: string;
    }>;

    $: graph = graphManager.getRootGraph();
    $: nodeStore = graph.nodeStore;

    $: {
        uiNodes = $nodeStore
            .filter(([node, _]) => node.data.state.value !== null)
            .map(([node, index]) => ({
                index,
                node: node.data,
                uiName: node.data.properties.ui_name.data as string,
            }));

        uiNodes.sort((a, b) => a.uiName.localeCompare(b.uiName));
    }

    function textareaUpdate(e: any) {
        if (currentlySelected) {
            let newValue = e.target.value as string;

            // very simple parsing
            const pairs = newValue
                .split("\n")
                .map((line) => {
                    line = line.trim();

                    if (line.length === 0) return undefined;

                    const splitPoint = line.indexOf("=");

                    const key = line.substring(0, splitPoint).trim();
                    let value = line.substring(splitPoint + 1).trim();

                    // if the value was in quotes, strip them
                    if (value.indexOf('"') === 0) {
                        value = value.substring(1, value.length - 1);
                    }

                    return [key, value];
                })
                .filter((x) => x !== undefined);

            const node = uiNodes.find(
                ({ index }) => index === currentlySelected?.nodeIndex
            );

            if (node?.node.uiData.panelInstances) {
                const element =
                    node.node.uiData.panelInstances["0"][
                        currentlySelected.elementIndex
                    ];

                element.properties = Object.fromEntries(
                    pairs as Array<[string, string]>
                );

                graph.markNodeAsUpdated(currentlySelected?.nodeIndex, [
                    "uiData",
                ]);
                graph.writeChangedNodesToServer();
            }
        }
    }

    function updateTextareaContent() {
        if (!currentlySelected) return;

        const node = uiNodes.find(
            ({ index }) => index === currentlySelected?.nodeIndex
        );

        if (node?.node.uiData.panelInstances) {
            const element =
                node.node.uiData.panelInstances["0"][
                    currentlySelected.elementIndex
                ];

            const joined = Object.entries(element.properties)
                .map(([key, value]) => `${key} = "${value}"`)
                .join("\n");

            textareaContent = joined;
        }
    }

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
            selected: false,
        };

        if (!node.uiData.panelInstances) {
            node.uiData.panelInstances = {
                "0": [],
            };
        }

        node.uiData.panelInstances["0"].push(newElementInstance);

        graph.markNodeAsUpdated(nodeIndex, ["uiData"]);
        graph.writeChangedNodesToServer();
    }

    function deselectAll() {
        for (let [index, node] of graph.getNodes()) {
            for (let instance of Object.values(
                node.uiData.panelInstances || {}
            )) {
                for (let element of instance) {
                    element.selected = false;
                }
            }

            graph.markNodeAsUpdated(index, ["uiData"]);
        }

        graph.writeChangedNodesToServer();

        currentlySelected = null;
        textareaContent = "";
    }

    function onNewPosition(
        nodeIndex: VertexIndex,
        elementIndex: number,
        x: number,
        y: number
    ) {
        const node = graph.getNode(nodeIndex);

        if (!node?.uiData.panelInstances) return;
        let element = node.uiData.panelInstances["0"][elementIndex];

        deselectAll();

        let didPositionChange = x !== element.x || y !== element.y;

        element.x = x;
        element.y = y;
        element.selected = true;

        currentlySelected = {
            nodeIndex: nodeIndex,
            instance: "0",
            elementIndex: elementIndex,
        };
        updateTextareaContent();

        graph.markNodeAsUpdated(nodeIndex, ["uiData"]);
    }

    function onSkinSelected(event: CustomEvent<string>) {
        const skinId = event.detail;

        if (currentlySelected !== null) {
            const node = graph.getNode(currentlySelected.nodeIndex);

            if (!node || !node.uiData.panelInstances) return;

            node.uiData.panelInstances[currentlySelected.instance][
                currentlySelected.elementIndex
            ].resourceId = skinId;

            graph.markNodeAsUpdated(currentlySelected.nodeIndex, ["uiData"]);
            graph.writeChangedNodesToServer();

            console.log(currentlySelected, skinId);
        }
    }

    $: if (locked) {
        deselectAll();
    }
</script>

<div>
    <div class="top-ui">
        Locked? <input type="checkbox" bind:checked={locked} />
    </div>
    <SplitView
        {width}
        height={height - 36}
        bind:firstHeight={viewportHeight}
        direction={SplitDirection.HORIZONTAL}
        initialSplitRatio={0.8}
    >
        <div slot="first">
            <SplitView
                {width}
                height={viewportHeight}
                direction={SplitDirection.VERTICAL}
                bind:firstWidth={draggableWidth}
                initialSplitRatio={0.2}
            >
                <div slot="first" class="container" on:dragstart={onDragStart}>
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
                    class="panel container"
                    slot="second"
                    on:dragover={onDragOver}
                    on:drop={onDrop}
                    style="position: relative; height: {viewportHeight}px"
                >
                    {#each uiNodes as { node, index, uiName } (index)}
                        {#if node.uiData["panelInstances"] && node.uiData.panelInstances["0"]}
                            {#each node.uiData.panelInstances["0"] as uiElement, elementIndex}
                                <UiElement
                                    resourceId={uiElement.resourceId}
                                    x={uiElement.x}
                                    y={uiElement.y}
                                    selected={uiElement.selected}
                                    properties={uiElement.properties}
                                    nodeType={node.nodeType}
                                    state={node.state}
                                    {locked}
                                    on:newposition={(e) =>
                                        onNewPosition(
                                            index,
                                            elementIndex,
                                            e.detail.x,
                                            e.detail.y
                                        )}
                                    on:newstate={(e) =>
                                        socket.updateNodeState([
                                            [index, e.detail],
                                        ])}
                                    {uiName}
                                    {globalState}
                                />
                            {/each}
                        {/if}
                    {/each}
                </div>
            </SplitView>
        </div>
        <div slot="second">
            <SplitView
                {width}
                height={height - viewportHeight - 20}
                direction={SplitDirection.VERTICAL}
            >
                <div slot="first" class="container">
                    {#each Object.keys($globalState.resources.ui) as resource}
                        <div style="display: inline-block; width: 80px">
                            <div style="position: relative; margin: 4px">
                                <UiElement
                                    resourceId={resource}
                                    nodeType="ToggleNode"
                                    state={{
                                        countedDuringMapset: false,
                                        value: false,
                                        other: undefined,
                                    }}
                                    properties={{}}
                                    locked={true}
                                    uiName="example"
                                    on:skinselected={onSkinSelected}
                                    {globalState}
                                />
                            </div>
                        </div>
                    {/each}
                </div>
                <div slot="second" class="container">
                    <textarea
                        on:input={textareaUpdate}
                        bind:value={textareaContent}
                    />
                </div>
            </SplitView>
        </div>
    </SplitView>
</div>

<style>
    textarea {
        width: 100%;
        height: 100%;
        resize: none;
        border: none;
        margin: 0;
        border-radius: 0;
    }

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

    .container {
        width: 100%;
        height: 100%;
        border-left: 1px solid black;
        border-top: 1px solid black;
    }

    .top-ui {
        padding: 8px;
        background-color: #ddd;
        height: 20px;
    }
</style>
