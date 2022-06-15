<script lang="ts">
    import { SocketType, SocketDirection, Primitive } from "../node-engine/connection";
    import { EnumInstance } from "../util/enum";

    export let direction: SocketDirection;
    export let type: EnumInstance/*SocketType*/;
    export let socketMousedown = function(event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection) {};
    export let socketMouseup = function(event: MouseEvent, socket: EnumInstance/*SocketType*/, direction: SocketDirection) {};

    function socketMousedownRaw(event: MouseEvent) {
        event.preventDefault();
        event.stopPropagation();

        socketMousedown(event, type, direction);
    }

    function socketMouseupRaw(event: MouseEvent) {
        socketMouseup(event, type, direction);
    }
</script>

<div class:output={direction === SocketDirection.Output} class:input={direction === SocketDirection.Input} class="socket-container">
    {#if type.getType() === SocketType.ids.Stream}
        <div class="socket stream" on:mousedown={socketMousedownRaw} on:mouseup={socketMouseupRaw}></div>
    {:else if type.getType() === SocketType.ids.Midi}
        <div class="socket midi" on:mousedown={socketMousedownRaw} on:mouseup={socketMouseupRaw}></div>
    {:else if type.getType() === SocketType.ids.Value}
        <div class="socket value" on:mousedown={socketMousedownRaw} on:mouseup={socketMouseupRaw}>
            <svg viewBox="0 0 26 26">
                <polygon points="13,1 25,25 1,25" />
            </svg>
        </div>
    {/if}
</div>


<style>
.socket-container {
    display: inline-block;
}
.input .socket {
    margin-left: -15px;
}

.output .socket {
    margin-right: -15px;
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
    width: 24px;
    height: 24px;
}

.value {
    fill: rgb(255, 166, 0);
    stroke: white;
}

.value polygon {
    fill: orange;
    stroke-width: 2;
    stroke: white;
}

</style>