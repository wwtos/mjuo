<script lang="ts">
    import { MemberType } from "safety-match";

    import { Graph } from "../node-engine/graph";
    import { NodeWrapper } from "../node-engine/node";
    import { Property, PropertyType } from "../node-engine/property";

    export let nodeWrapper: NodeWrapper;
    export let propName: string;
    export let propType: MemberType<typeof PropertyType>;
    export let nodes: Graph;

    let choices: any = propType.data;

    let value = nodeWrapper.getPropertyValue(propName);

    function updateProperties(event) {
        const newValue = event.target.value;
        
        const newValueParsed = propType.match({
            MultipleChoice: (_) => {
                event.target.value = newValue;

                return Property.MultipleChoice(newValue);
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
        <select value={$value.data} on:input={updateProperties}>
            {#each choices as choice (choice)}
                <option value={ choice }>{ choice }</option>
            {/each}
        </select>
    {/if}
</div>

<style>
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