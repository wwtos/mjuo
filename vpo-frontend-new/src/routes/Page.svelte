<script lang="ts">
    import { FluentBundle, FluentResource } from "@fluent/bundle";
    import { FluentProvider } from "@nubolab-ffwd/svelte-fluent";

    import enTranslations from "$lib/assets/lang/en.ftl?raw";

    const bundle = new FluentBundle("en");
    bundle.addResource(new FluentResource(enTranslations));
</script>

<FluentProvider bundles={[bundle]}>
    <slot />
</FluentProvider>
