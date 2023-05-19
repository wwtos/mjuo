<script lang="ts">
    import type { VertexIndex } from "$lib/ddgg/graph";
    import type { NodeWrapper } from "$lib/node-engine/node";
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import type { Property, PropertyType } from "$lib/node-engine/property";
    import { matchOrElse } from "$lib/util/discriminated-union";
    import { localize } from "@nubolab-ffwd/svelte-fluent";
    import { deepEqual } from "fast-equals";
    import { preventHistoryKeyActions } from "./editor-utils";

    export let nodeWrapper: NodeWrapper;
    export let nodeIndex: VertexIndex;
    export let propName: string;
    export let propType: PropertyType;
    export let nodes: NodeGraph;
    export let value: Property;

    function updateProperties(this: HTMLSelectElement, event: Event) {
        const newValue = this.value;

        const newValueParsed = matchOrElse(
            propType,
            {
                MultipleChoice: (_): Property => {
                    this.value = newValue;

                    return { variant: "MultipleChoice", data: newValue };
                },
                Integer: (): Property => {
                    const newValueParsed = parseInt(newValue);
                    this.value = newValueParsed + "";

                    return { variant: "Integer", data: newValueParsed };
                },
                String: (): Property => {
                    return { variant: "String", data: newValue };
                },
                Resource: (): Property => {
                    return {
                        variant: "Resource",
                        data: {
                            namespace: dataAsResource.namespace,
                            resource: newValue,
                        },
                    };
                },
            },
            () => {
                throw new Error("unimplemened");
            }
        );

        // only send updates if it's changed
        if (!deepEqual(nodeWrapper.properties[propName], newValueParsed)) {
            nodeWrapper.properties[propName] = newValueParsed;

            nodes.updateNode(nodeIndex);
            nodes.markNodeAsUpdated(nodeIndex);
            nodes.writeChangedNodesToServer();
        }
    }

    $: dataAsResource = value?.data as { namespace: string; resource: string };
    $: dataAsAny = value?.data as any;
</script>

<div class="container">
    {#if propType.variant === "MultipleChoice"}
        <select
            value={value.data}
            on:mousedown={(e) => e.stopPropagation()}
            on:input={updateProperties}
        >
            {#each propType.data as choice (choice)}
                <option value={choice}>{choice}</option>
            {/each}
        </select>
    {:else if propType.variant == "Integer"}
        <div class="flex">
            <label>
                <input
                    type="number"
                    value={value.data}
                    on:mousedown={(e) => e.stopPropagation()}
                    on:change={updateProperties}
                    on:keydown={preventHistoryKeyActions}
                />
                <div>
                    <span class="input-hover-text"
                        >{$localize("property." + propName)}</span
                    >
                </div>
            </label>
        </div>
    {:else if propType.variant == "String"}
        <div class="flex">
            <label>
                <input
                    type="text"
                    value={value.data}
                    title={$localize("property." + propName)}
                    on:mousedown|stopPropagation
                    on:change={updateProperties}
                    on:keydown={preventHistoryKeyActions}
                />
                {#if dataAsAny.length < 15}
                    <div>
                        <span class="input-hover-text"
                            >{$localize("property." + propName)}</span
                        >
                    </div>
                {/if}
            </label>
        </div>
    {:else if propType.variant == "Resource"}
        <div class="flex">
            <label>
                {dataAsResource.namespace}:&nbsp;
                <input
                    type="text"
                    value={dataAsResource.resource}
                    title={$localize("property." + propName)}
                    on:mousedown|stopPropagation
                    on:change={updateProperties}
                    on:keydown={preventHistoryKeyActions}
                />
                {#if (dataAsResource.namespace + ":" + dataAsResource.resource).length < 15}
                    <div>
                        <span class="input-hover-text"
                            >{$localize("property." + propName)}</span
                        >
                    </div>
                {/if}
            </label>
        </div>
    {/if}
</div>

<style>
    label {
        position: relative;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: text;
        width: calc(100% - 40px);
    }

    label > div {
        position: absolute;
        width: 100%;
        display: flex;
        justify-content: flex-end;
        flex-direction: row;
    }

    label > input {
        margin: 0;
        width: 100%;
    }

    label > div > span {
        color: #777;
        margin: 0 12px;
    }

    .flex {
        display: flex;
        flex-flow: column;
        align-items: center;
    }

    input {
        border-radius: 5px;
        height: 26px;
        border: none;
        outline: none;
        box-shadow: none;
        resize: none;
    }

    input:focus-visible {
        outline: 1px solid blue;
    }

    .container {
        margin: 10px 0;
        height: 26px;
    }

    select {
        border-radius: 5px;
        width: calc(100% - 32px);
        margin: 0 16px;
        background: white;
        height: 26px;
        padding: 2px;
        border: none;
        outline: none;
        box-shadow: none;
        resize: none;
    }

    select:focus-visible {
        outline: 1px solid blue;
    }
</style>
