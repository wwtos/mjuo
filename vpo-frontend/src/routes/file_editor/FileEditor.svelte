<script lang="ts">
    import type { IpcSocket } from "$lib/ipc/socket";
    import type { GlobalState } from "$lib/node-engine/global_state";
    import type { Writable } from "svelte/store";

    export let globalState: Writable<GlobalState>;
    export let socket: IpcSocket;

    let fileInput: HTMLInputElement;

    function openFileViewer() {
        socket.create();
    }
</script>

<div style="padding: 8px">
    <label>
        <button on:click={openFileViewer}>Create project</button>
    </label>

    {#if $globalState.activeProject}
        <h1>{$globalState.activeProject}</h1>
    {/if}
</div>
