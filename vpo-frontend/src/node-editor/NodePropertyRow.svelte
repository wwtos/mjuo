<script lang="ts">
    import { MemberType } from "safety-match";

    import { NodeGraph } from "../node-engine/node_graph";
    import { NodeWrapper } from "../node-engine/node";
    import { Property, PropertyType } from "../node-engine/property";
    import { matchOrElse } from "../util/discriminated-union";
    import { dataset_dev } from "svelte/internal";
    import { deepEqual } from "fast-equals";

    export let nodeWrapper: NodeWrapper;
    export let propName: string;
    export let propType: PropertyType;
    export let nodes: NodeGraph;

    let value = nodes.getNodePropertyValue(nodeWrapper.index, propName);

    function updateProperties(event) {
        const newValue = event.target.value;
        console.log(newValue);

        const newValueParsed = matchOrElse(
            propType,
            {
                MultipleChoice: (_): Property => {
                    event.target.value = newValue;

                    return { variant: "MultipleChoice", data: newValue };
                },
                Integer: (): Property => {
                    const newValueParsed = parseInt(newValue);
                    event.target.value = newValueParsed;

                    return { variant: "Integer", data: newValueParsed };
                },
                String: (): Property => {
                    return { variant: "String", data: newValue };
                },
                Resource: (): Property => {
                    let parts = newValue.split(":");
                    let namespace = parts[0];
                    let resource = parts.slice(1).join(":");

                    return {
                        variant: "Resource",
                        data: {
                            namespace,
                            resource,
                        },
                    };
                },
            },
            () => {
                throw new Error("unimplemened");
            }
        );

        console.log(nodeWrapper.properties[propName], newValueParsed);

        // only send updates if it's changed
        if (!deepEqual(nodeWrapper.properties[propName], newValueParsed)) {
            nodeWrapper.properties[propName] = newValueParsed;

            nodes.updateNode(nodeWrapper.index);
            nodes.markNodeAsUpdated(nodeWrapper.index);
            nodes.writeChangedNodesToServer();
        }
    }

    $: dataAsResource = $value.data as { namespace: string; resource: string };
</script>

<div class="container">
    {#if propType.variant === "MultipleChoice"}
        <select
            value={$value.data}
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
                    value={$value.data}
                    on:mousedown={(e) => e.stopPropagation()}
                    on:change={updateProperties}
                    on:keydown={(event) => event.stopPropagation()}
                />
                <div>
                    <span class="input-hover-text">{propName}</span>
                </div>
            </label>
        </div>
    {:else if propType.variant == "String"}
        <div class="flex">
            <label>
                <input
                    type="text"
                    value={$value.data}
                    on:mousedown={(e) => e.stopPropagation()}
                    on:change={updateProperties}
                    on:keydown={(event) => event.stopPropagation()}
                />
                <div>
                    <span class="input-hover-text">{propName}</span>
                </div>
            </label>
        </div>
    {:else if propType.variant == "Resource"}
        <div class="flex">
            <label>
                <input
                    type="text"
                    value={dataAsResource.namespace +
                        ":" +
                        dataAsResource.resource}
                    on:mousedown={(e) => e.stopPropagation()}
                    on:change={updateProperties}
                    on:keydown={(event) => event.stopPropagation()}
                />
                <div>
                    <span class="input-hover-text">{propName}</span>
                </div>
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
    }
</style>
