<script lang="ts">
    import { onMount } from "svelte";

    import { SocketType, SocketDirection, Primitive } from "../node-engine/connection";
    import { NodeWrapper } from "../node-engine/node";
    import { EnumInstance } from "../util/enum";

    import Socket from "./Socket.svelte";

    const RADIUS = 12;

    export let type: EnumInstance/*SocketType*/;
    export let label: string;
    export let direction: SocketDirection;
    export let nodeWrapper: NodeWrapper;
    export let socketMousedown = function(event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection) {};
    export let socketMouseup = function(event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection) {};
    export let defaultValue;

    console.log(defaultValue, defaultValue.enumDef === Primitive);
    
    let socket: HTMLDivElement;
</script>
<div class="container" class:output={direction === SocketDirection.Output} class:input={direction === SocketDirection.Input}>
    {#if direction === SocketDirection.Input}
        <Socket {direction} {type} {socketMousedown} {socketMouseup} />
    {/if}

    {#if type.getType() === SocketType.ids.Value && direction === SocketDirection.Input && !nodeWrapper.getInputConnectionByType(type) }
        {#if defaultValue.getType() === Primitive.ids.Float}
            <div class="flex">
                <label>
                    <input />
                    <span class="input-hover-text">{ label }</span>
                </label>
            </div>
        {:else if defaultValue.getType() === Primitive.ids.Boolean}
            <input type="checkbox" />
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