<script lang="ts">
    import { createEventDispatcher } from "svelte";

    import { SocketType, SocketDirection } from "$lib/node-engine/connection";

    const dispatch = createEventDispatcher();

    export let direction: SocketDirection;
    export let type: SocketType;

    function socketMousedown(event: MouseEvent) {
        event.preventDefault();
        event.stopPropagation();

        dispatch("socketMousedown", {
            event,
            type,
            direction,
        });
    }

    function socketMouseupRaw(event: MouseEvent) {
        dispatch("socketMouseup", {
            event,
            type,
            direction,
        });
    }
</script>

<div
    class:output={direction === SocketDirection.Output}
    class:input={direction === SocketDirection.Input}
    class="socket-container"
>
    {#if type.variant === "Stream"}
        <div
            class="socket stream"
            on:mousedown={socketMousedown}
            on:mouseup={socketMouseupRaw}
        />
    {:else if type.variant === "Midi"}
        <div
            class="socket midi"
            on:mousedown={socketMousedown}
            on:mouseup={socketMouseupRaw}
        />
    {:else if type.variant === "Value"}
        <div
            class="socket value"
            on:mousedown={socketMousedown}
            on:mouseup={socketMouseupRaw}
        >
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
        background: rgb(231, 200, 59);
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
