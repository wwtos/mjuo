<script lang="ts">
import { Graph } from "../node-engine/graph";
import { NodeWrapper } from "../node-engine/node";
import { Property, PropertyType } from "../node-engine/property";
import { IPCSocket } from "../util/socket";
import FloatProperty from "./FloatProperty.svelte";

import NumberProperty from "./NumberProperty.svelte";
import OptionsProperty from "./OptionsProperty.svelte";
import StringProperty from "./StringProperty.svelte";

export let width: number;
export let height: number;

export let ipcSocket: IPCSocket;
export let nodes: Graph;

let selected: NodeWrapper;

nodes.nodeStore.subscribe(nodes => {
    selected = nodes.find(node => node.uiData.selected);
});

function changeSelectedUIData(prop, content) {
    // if the content is the same, disregard (to stop an infinite network update loop)
    if (selected.uiData[prop] === content) return;

    if (selected) {
        selected.uiData[prop] = content;

        nodes.markNodeAsUpdated(selected.index);
        nodes.writeChangedNodesToServer();

        nodes.update();
    }
}

function changeSelectedProp(prop, content) {
    // if the content is the same, disregard (to stop an infinite network update loop)
    if (selected.properties[prop] && selected.properties[prop].content === content) return;

    if (selected.properties[prop] &&
        selected.properties[prop].type === PropertyType.ids.Float &&
        Math.abs(content - selected.properties[prop].content) < 0.00001) return;

    if (selected) {
        // create an enum of that type
        if (!selected.properties[prop]) {
            selected.properties[prop] = Property[selected.node.usableProperties[prop].toName()](content);
        } else {
            selected.properties[prop].content = content;
        }

        ipcSocket.updateNodes([selected]);
    }
}
</script>

<div style="width: {width}px; height: {height}px">
    {#if selected}
        <div class="container">
            <div class="row">
                <div class="prop-name">Title</div>
                <div class="horizontal-divide"></div>
                <div class="prop-value">
                    <StringProperty value={selected.uiData.title} onchange={changeSelectedUIData.bind(null, "title")} />
                </div>
            </div>
            {#each Object.entries(selected.node.usableProperties) as [id, propertyType] }
                <div class="row">
                    <div class="prop-name">{id}</div>
                    <div class="horizontal-divide"></div>
                    {#if propertyType.type === PropertyType.ids.Float}
                        <div class="prop-value">
                            <FloatProperty
                                value={selected.properties[id] !== undefined ? selected.properties[id].content : 0 } 
                                step={0.01}
                                onchange={changeSelectedProp.bind(null, id)}
                            />
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>    
.horizontal-divide {
    flex-grow: 0;
    border-left: 1px solid rgb(216, 216, 216);
}

.container {
    display: flex;
    flex-direction: column;
}

.row {
    width: 100%;
    display: inline-flex;
    flex-direction: row;
    justify-content: center;
    padding: 0;
    border-bottom: 1px solid rgb(216, 216, 216);
}

.row:hover {
    background-color: rgb(223, 223, 223);
}

.prop-name, .prop-value {
    width: 50%;
    margin: 0;
}

.prop-name {
    text-align: right;
    padding: 3px 6px 3px 5px;
}

.prop-value {
    padding: 0;
}
</style>