<script lang="ts">
    import { page } from "$app/stores";
    import { createEventDispatcher } from "svelte";
    import {
        FolderIcon,
        GitPullRequestIcon,
        LayersIcon,
        SettingsIcon,
    } from "svelte-feather-icons";

    export let section: string;

    const dispatch = createEventDispatcher();

    function onClick(section: string) {
        return () => {
            dispatch("click", section);
        };
    }

    export let width = 48;
</script>

<nav style="min-width: {width}px">
    <div>
        <ul>
            <li
                class:active={section === "nodeEditor"}
                on:mousedown={onClick("nodeEditor")}
            >
                <a href={null}>
                    <span style="padding-left: 2px">
                        <GitPullRequestIcon size="30" strokeWidth={1.5} />
                    </span>
                </a>
            </li>
            <li
                class:active={section === "uiEditor"}
                on:mousedown={onClick("uiEditor")}
            >
                <a href={null}>
                    <span style="padding-left: 2px">
                        <LayersIcon size="30" strokeWidth={1.5} />
                    </span>
                </a>
            </li>
            <li
                class:active={section === "fileEditor"}
                on:mousedown={onClick("fileEditor")}
            >
                <a href={null}>
                    <span style="padding-left: 2px">
                        <FolderIcon size="30" strokeWidth={1.5} />
                    </span>
                </a>
            </li>
            <li
                class:active={section === "settings"}
                on:mousedown={onClick("settings")}
            >
                <a href={null}>
                    <span style="padding-left: 2px">
                        <SettingsIcon size="30" strokeWidth={1.5} />
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
