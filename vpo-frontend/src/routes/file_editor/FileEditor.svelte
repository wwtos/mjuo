<script lang="ts">
    import type { IpcSocket } from "$lib/ipc/socket";
    import type { GlobalState } from "$lib/node-engine/global_state";
    import type { Writable } from "svelte/store";

    export let globalState: Writable<GlobalState>;
    export let socket: IpcSocket;

    let fileInput: HTMLInputElement;
    let importingRank: boolean;
    let rankFileName: string = "";
    let rankName: string = "";

    function openFileViewer() {
        socket.create();
    }

    function importRankFiles() {
        socket.importRank(rankFileName, rankName);
    }
</script>

<div style="padding: 8px">
    <label>
        <button on:click={openFileViewer}>Create project</button>
    </label>

    {#if $globalState.activeProject}
        <h1>{$globalState.activeProject}</h1>
        <button on:click={() => (importingRank = !importingRank)}
            >Import rank</button
        >
        {#if importingRank}
            <label>
                Rank file name:
                <input bind:value={rankFileName} />
            </label>
            <label>
                Rank name:
                <input bind:value={rankName} />
            </label>
            <button on:click={importRankFiles}
                >Import files (fill out other fields first)</button
            >
        {/if}
    {/if}
</div>
