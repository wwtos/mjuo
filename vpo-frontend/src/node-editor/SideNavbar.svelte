<script>
    import {
        FolderIcon,
        GitPullRequestIcon,
        LayersIcon,
    } from "svelte-feather-icons";

    let active = 0;
</script>

<nav>
    <ul>
        <li class:active={active === 0} on:click={() => (active = 0)}>
            <span style="padding-left: 2px">
                <GitPullRequestIcon size="30" strokeWidth="1.5" />
            </span>
        </li>
        <li class:active={active === 1} on:click={() => (active = 1)}>
            <span style="padding-left: 2px">
                <LayersIcon size="30" strokeWidth="1.5" />
            </span>
        </li>
        <li class:active={active === 2} on:click={() => (active = 2)}>
            <span style="padding-left: 2px">
                <FolderIcon size="30" strokeWidth="1.5" />
            </span>
        </li>
    </ul>
</nav>

<style>
    nav {
        width: 48px !important;
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
