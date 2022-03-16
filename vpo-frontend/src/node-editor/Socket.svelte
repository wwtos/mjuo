<script lang="ts">
import { onMount } from "svelte";

    import { SocketType, SocketDirection } from "../node-engine/connection";
    import { EnumInstance } from "../util/enum";

    const RADIUS = 12;

    export let type: EnumInstance/*SocketType*/;
    export let label: string;
    export let direction: SocketDirection;
    export let socketMousedown = function(event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection) {};
    export let socketMouseup = function(event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection) {};
    export let exportSocketPosition = function(socket: EnumInstance/*SocketType*/, direction: SocketDirection, rect: DOMRect) {};
    
    let socket: HTMLDivElement;

    onMount(async () => {
        exportSocketPosition(type, direction, socket.getBoundingClientRect());
    });

    function socketMousedownRaw(event: MouseEvent) {
        event.preventDefault();
        event.stopPropagation();

        socketMousedown(event, type, direction);
    }

    function socketMouseupRaw(event: MouseEvent) {
        socketMouseup(event, type, direction);
    }
</script>
<div class="container" class:output={direction === SocketDirection.Output} class:input={direction === SocketDirection.Input}>
    <!-- put the text first if it's an output -->
    {#if direction === SocketDirection.Output}
        <div class="text">{ label }</div>
    {/if}

    {#if type.getType() === SocketType.ids.Stream}
        <div class="socket stream" on:mousedown={socketMousedownRaw} on:mouseup={socketMouseupRaw} bind:this={socket}></div>
    {:else if type.getType() === SocketType.ids.Midi}
        <div class="socket midi" on:mousedown={socketMousedownRaw} on:mouseup={socketMouseupRaw} bind:this={socket}></div>
    {:else if type.getType() === SocketType.ids.Value}
        <div class="socket value" on:mousedown={socketMousedownRaw} on:mouseup={socketMouseupRaw} bind:this={socket}>
            <svg viewBox="0 0 26 26">
                <polygon points="13,1 25,25 1,25" />
            </svg>
        </div>
    {/if}

    {#if direction === SocketDirection.Input}
        <div class="text">{ label }</div>
    {/if}
</div>

<style>
.value polygon {
    fill: orange;
    stroke-width: 2;
    stroke: white;
}
.container {
    margin: 10px 0;
}

.input {
    text-align: left;
}

.output {
    text-align: right;
}

.input .socket {
    margin-left: -15px;
}

.output .socket {
    margin-right: -15px;
}


.text {
    display: inline-block;
    color: white;
}

.socket {
    width: 26px;
    height: 26px;
    vertical-align: middle;
    display: inline-block;
}

.stream {
    border-radius: 100%;
    background: #96b38a;
    border: 2px solid white;
    width: 22px;
    height: 22px;
}

.midi {
    background: gold;
    border: 2px solid white;
}

.value {
    fill: rgb(255, 166, 0);
    stroke: white;
}
</style>