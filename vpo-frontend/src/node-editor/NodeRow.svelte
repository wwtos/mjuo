<script lang="ts">
    import { onMount } from "svelte";

    import { SocketType, SocketDirection, Primitive, ValueSocketType, areSocketTypesEqual } from "../node-engine/connection";
    import { NodeRow, NodeWrapper } from "../node-engine/node";
    import { Graph } from "../node-engine/graph";
    import { EnumInstance } from "../util/enum";
    import { fixDigits } from "../util/fix-digits";
    import { BehaviorSubject } from "rxjs";

    import Socket from "./Socket.svelte";

    const RADIUS = 12;

    export let type: EnumInstance/*SocketType*/;
    export let label: string;
    export let direction: SocketDirection;
    export let nodeWrapper: NodeWrapper;
    export let socketMousedown = function(event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection) {};
    export let socketMouseup = function(event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection) {};
    export let defaultValue;
    export let nodes: Graph;
    
    let socket: HTMLDivElement;

    let isConnected = direction === SocketDirection.Input ? nodeWrapper.getInputConnectionByType(type) : new BehaviorSubject(undefined);

    let socketDefault = new BehaviorSubject(undefined);
    nodeWrapper.getSocketDefault(type, direction).subscribe(socketDefault);

    function updateOverrides(event) {
        const newValue = event.target.value;
        
        const newValueParsed = type.match([
            [SocketType.ids.Stream, () => {
                const num = parseFloat(newValue);
                event.target.value = num;

                return isNaN(num) ? 0.0 : num;
            }],
            [SocketType.ids.Value, valueType => {
                return socketDefault.getValue().match([
                    [Primitive.ids.Float, _ => {
                        const num = parseFloat(newValue);
                        event.target.value = num;

                        return Primitive.Float(isNaN(num) ? 0.0 : num);
                    }],
                    // booleans are special
                    [Primitive.ids.Boolean, _ => {
                        return Primitive.Boolean(event.target.checked);
                    }]
                ]);
            }]
        ]);
        
        // check if this override is already in there, in which case the value needs to be updated
        let override = nodeWrapper.defaultOverrides.getValue().find(defaultOverride => {
            const [overrideSocketType, overrideDirection] = NodeRow.asTypeAndDirection(defaultOverride);

            return areSocketTypesEqual(type, overrideSocketType) &&
                   direction === overrideDirection;
        });

        if (override) {
            override.content[1] = newValueParsed;
        } else {
            nodeWrapper.defaultOverrides.next([
                ...nodeWrapper.defaultOverrides.getValue(),
                NodeRow.fromTypeAndDirection(type, direction, newValueParsed)
            ]);
        }

        nodes.markNodeAsUpdated(nodeWrapper.index);
        nodes.writeChangedNodesToServer();
    }
</script>
<div class="container" class:output={direction === SocketDirection.Output} class:input={direction === SocketDirection.Input}>
    {#if direction === SocketDirection.Input}
        <Socket {direction} {type} {socketMousedown} {socketMouseup} />
    {/if}

    {#if direction === SocketDirection.Input && !$isConnected}
        {#if type.getType() === SocketType.ids.Value}
            {#if defaultValue.getType() === Primitive.ids.Float}
                <div class="flex">
                    <label>
                        <input value={fixDigits(($socketDefault).content, 3)} on:change={updateOverrides} on:keydown={event => event.stopPropagation()} />
                        <span class="input-hover-text">{ label }</span>
                    </label>
                </div>
            {:else if defaultValue.getType() === Primitive.ids.Boolean}
                <input type="checkbox" on:change={updateOverrides} checked={($socketDefault).content} />
                <div class="text">{ label }</div>
            {/if}
        {:else if type.getType() === SocketType.ids.Stream}
            <div class="flex">
                <label>
                    <input value={fixDigits($socketDefault, 3)} on:change={updateOverrides} on:keydown={event => event.stopPropagation()} />
                    <span class="input-hover-text">{ label }</span>
                </label>
            </div>
        {:else}
        <div class="text">{ label }</div>
        {/if}
    {:else}
        <div class="text">{ label }</div>
    {/if}

    {#if direction === SocketDirection.Output}
        <Socket {direction} {type} {socketMousedown} {socketMouseup} />
    {/if}
</div>

<style>
label {
    position: relative;
    display: flex;
    align-items: center;
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
}

.flex {
    display: flex;
    flex-flow: column;
    align-items: center;
    margin-top: -12px;
}

input {
    border-radius: 5px;
}

input[type="checkbox"] {
    height: initial;
}

.container {
    margin: 10px 0;
    height: 26px;
}

.input {
    text-align: left;
}

.output {
    text-align: right;
}

.text {
    display: inline-block;
    color: white;
}
</style>