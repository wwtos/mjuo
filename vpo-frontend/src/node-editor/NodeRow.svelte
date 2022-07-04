<script lang="ts">
    import { onMount } from "svelte";

    import { SocketType, SocketDirection, Primitive, ValueSocketType, areSocketTypesEqual } from "../node-engine/connection";
    import { NodeRow, NodeWrapper, NodeRowAsTypeAndDirection, NodeRowFromTypeAndDirection } from "../node-engine/node";
    import { Graph } from "../node-engine/graph";
    import { fixDigits } from "../util/fix-digits";
    import { BehaviorSubject, Observable } from "rxjs";

    import Socket from "./Socket.svelte";
    import { MemberType } from "safety-match";

    const RADIUS = 12;

    export let type: MemberType<typeof SocketType>;
    export let label: BehaviorSubject<string>;
    export let direction: SocketDirection;
    export let nodeWrapper: NodeWrapper;
    export let socketMousedown = function(event: MouseEvent, socket: MemberType<typeof SocketType>, direction: SocketDirection) {};
    export let socketMouseup = function(event: MouseEvent, socket: MemberType<typeof SocketType>, direction: SocketDirection) {};
    export let defaultValue;
    export let nodes: Graph;
    
    let socket: HTMLDivElement;

    let isConnected = direction === SocketDirection.Input ? nodeWrapper.getInputConnectionByType(type) : new BehaviorSubject(undefined);

    let socketDefault = new BehaviorSubject(undefined);
    nodeWrapper.getSocketDefault(type, direction).subscribe(socketDefault);

    function updateOverrides(event) {
        const newValue = event.target.value;
        
        const newValueParsed = type.match({
            Stream: () => {
                const num = parseFloat(newValue);
                event.target.value = num;

                return isNaN(num) ? 0.0 : num;
            },
            Value: valueType => {
                return socketDefault.getValue().match({
                    Float: _ => {
                        const num = parseFloat(newValue);
                        event.target.value = num;

                        return Primitive.Float(isNaN(num) ? 0.0 : num);
                    },
                    // booleans are special
                    Boolean: _ => {
                        return Primitive.Boolean(event.target.checked);
                    }
                });
            },
            _: () => { throw "unimplemented" }
        });
        
        // check if this override is already in there, in which case the value needs to be updated
        let override = nodeWrapper.defaultOverrides.getValue().find(defaultOverride => {
            const [overrideSocketType, overrideDirection] = NodeRowAsTypeAndDirection(defaultOverride);

            return areSocketTypesEqual(type, overrideSocketType) &&
                   direction === overrideDirection;
        });

        if (override) {
            override.data[1] = newValueParsed;
        } else {
            nodeWrapper.defaultOverrides.next([
                ...nodeWrapper.defaultOverrides.getValue(),
                NodeRowFromTypeAndDirection(type, direction, newValueParsed)
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
        {#if type.variant === "Value"}
            {#if defaultValue.variant === "Float"}
                <div class="flex">
                    <label>
                        <input value={fixDigits(($socketDefault).data, 3)} on:mousedown={e => e.stopPropagation()} on:change={updateOverrides} on:keydown={event => event.stopPropagation()} />
                        <span class="input-hover-text">{ $label }</span>
                    </label>
                </div>
            {:else if defaultValue.variant === "Boolean"}
                <input type="checkbox" on:change={updateOverrides} on:mousedown={e => e.stopPropagation()} checked={($socketDefault).data} />
                <div class="text">{ $label }</div>
            {/if}
        {:else if type.variant === "Stream"}
            <div class="flex">
                <label>
                    <input value={fixDigits($socketDefault, 3)} on:mousedown={e => e.stopPropagation()} on:change={updateOverrides} on:keydown={event => event.stopPropagation()} />
                    <span class="input-hover-text">{ $label }</span>
                </label>
            </div>
        {:else}
        <div class="text">{ $label }</div>
        {/if}
    {:else}
        <div class="text">{ $label }</div>
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