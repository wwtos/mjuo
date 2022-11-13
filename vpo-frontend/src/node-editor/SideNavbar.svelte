<script>
    import {
        FolderIcon,
        GitPullRequestIcon,
        LayersIcon,
    } from "svelte-feather-icons";

    import { activeEditor } from "./state";

    export let width = 48;
</script>

<nav style="min-width: {width}px">
    <div>
        <ul>
            <li
                class:active={$activeEditor === "nodes"}
                on:click={() => activeEditor.next("nodes")}
            >
                <span style="padding-left: 2px">
                    <GitPullRequestIcon size="30" strokeWidth="1.5" />
                </span>
            </li>
            <li
                class:active={$activeEditor === "ui"}
                on:click={() => activeEditor.next("ui")}
            >
                <span style="padding-left: 2px">
                    <LayersIcon size="30" strokeWidth="1.5" />
                </span>
            </li>
            <li
                class:active={$activeEditor === "files"}
                on:click={() => activeEditor.next("files")}
            >
                <span style="padding-left: 2px">
                    <FolderIcon size="30" strokeWidth="1.5" />
                </span>
            </li>
        </ul>
    </div>
</nav>

<style>
    nav {
        height: 100vh;
        background-color: #333333;
    }

    ul {
        padding: 0;
        margin: 0;
        list-style: none;
    }

    li {
        margin: 0;
        padding: 8px 0 6px 8px;
        color: grey;
        cursor: pointer;
    }

    li:hover {
        color: #eee;
    }

    .active {
        color: #eee;
        border-left: 2px solid #eee;
        padding-left: 6px;
    }
</style>
