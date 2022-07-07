<script lang="ts">
    import { MemberType } from "safety-match";

    import { NodeGraph } from "../node-engine/node_graph";
    import { NodeWrapper } from "../node-engine/node";
    import { Property, PropertyType } from "../node-engine/property";

    export let nodeWrapper: NodeWrapper;
    export let propName: string;
    export let propType: MemberType<typeof PropertyType>;
    export let nodes: NodeGraph;

    let choices: any = propType.data;

    let value = nodeWrapper.getPropertyValue(propName);

    function updateProperties(event) {
        const newValue = event.target.value;
        
        const newValueParsed = propType.match({
            MultipleChoice: (_) => {
                event.target.value = newValue;

                return Property.MultipleChoice(newValue);
            },
            Integer: () => {
                const newValueParsed = parseInt(newValue);
                event.target.value = newValueParsed;

                return Property.Integer(newValueParsed);
            },
            String: () => {
                return Property.String(newValue);
            },
            _: () => { throw "unimplemened" }
        });

        nodeWrapper.properties.next({
            ...nodeWrapper.properties.getValue(),
            [propName]: newValueParsed
        });

        nodes.markNodeAsUpdated(nodeWrapper.index);
        nodes.writeChangedNodesToServer();
    }
</script>

<div class="container">
    {#if propType.variant === "MultipleChoice"}
        <select value={$value.data} on:mousedown={e => e.stopPropagation()} on:input={updateProperties}>
            {#each choices as choice (choice)}
                <option value={ choice }>{ choice }</option>
            {/each}
        </select>
    {:else if propType.variant == "Integer"}
        <div class="flex">
            <label>
                <input type="number" value={$value.data} on:mousedown={e => e.stopPropagation()} on:change={updateProperties} on:keydown={event => event.stopPropagation()} />
                <span class="input-hover-text">{ propName }</span>
            </label>
        </div>
    {:else if propType.variant == "String"}
        <div class="flex">
            <label>
                <input type="text" value={$value.data} on:mousedown={e => e.stopPropagation()} on:change={updateProperties} on:keydown={event => event.stopPropagation()} />
                <span class="input-hover-text">{ propName }</span>
            </label>
        </div>
    {/if}
</div>

<style>
label {
    position: relative;
    display: flex;
    justify-content: center;
    cursor: text;
}
label>span,
label>input {
    position: absolute;
    margin: 0;
}

label>span {
    color: #777;
    margin-right: -80px;
    margin-top: 5px;
}

.flex {
    display: flex;
    flex-flow: column;
    height: 26px;
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