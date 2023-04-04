<script>
    import { page } from "$app/stores";
    import {
        FolderIcon,
        GitPullRequestIcon,
        LayersIcon,
    } from "svelte-feather-icons";

    export let width = 48;
</script>

<nav style="min-width: {width}px">
    <div>
        <ul>
            <li class:active={$page.url.pathname === "/node_editor"}>
                <a href="/node_editor">
                    <span style="padding-left: 2px">
                        <GitPullRequestIcon size="30" strokeWidth={1.5} />
                    </span>
                </a>
            </li>
            <li class:active={$page.url.pathname === "/ui_editor"}>
                <a href="/ui_editor">
                    <span style="padding-left: 2px">
                        <LayersIcon size="30" strokeWidth={1.5} />
                    </span>
                </a>
            </li>
            <li class:active={$page.url.pathname === "/file_editor"}>
                <a href="/file_editor">
                    <span style="padding-left: 2px">
                        <FolderIcon size="30" strokeWidth={1.5} />
                    </span>
                </a>
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

    a {
        color: inherit;
    }
</style>
