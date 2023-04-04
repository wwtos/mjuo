<script>
    import Page from "./Page.svelte";

    import { Localized } from "@nubolab-ffwd/svelte-fluent";
</script>

<Page>
    <h1>Welcome to SvelteKit</h1>
    <p>
        Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation
    </p>
    <p>
        Localization test: <Localized
            id="socket-input-numbered"
            args={{ id: 1 }}
        />
    </p>
</Page>
