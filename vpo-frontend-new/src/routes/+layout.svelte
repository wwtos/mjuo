<script lang="ts">
    import { FluentBundle, FluentResource } from "@fluent/bundle";
    import { FluentProvider } from "@nubolab-ffwd/svelte-fluent";

    import enTranslations from "$lib/assets/lang/en.ftl?raw";
    import SideNavbar from "./node_editor/SideNavbar.svelte";
    import Toasts from "$lib/components/Toasts.svelte";
    import type { PageData } from "./$types";
    import type { IpcAction } from "$lib/ipc/action";
    import { constructEngine } from "./engine";
    import { onMount } from "svelte";

    import { fetchSharedBuffer } from "$lib/util/fetch-shared-buffer";

    export let data: PageData;

    const bundle = new FluentBundle("en");
    bundle.addResource(new FluentResource(enTranslations));

    const context = new AudioContext();

    constructEngine(context).then((engine) => {
        data.socket.setEngine(engine);
    });

    function getOrgan(organ: string) {
        return fetch(`/${organ}/resources.json`).then((data) => data.json());
    }

    function onWindowKeydown(event: KeyboardEvent) {
        if (event.ctrlKey) {
            switch (event.key) {
                case "s":
                    data.socket.save();
                    break;
                case "o":
                    data.socket.load();
                    break;
            }
        }
    }

    async function onWindowClick() {
        if (context.state !== "running") {
            await context.resume();
            data.socket.flushMessages();
        }
    }

    function registerSocketEvents() {
        data.socket.onMessage((message: IpcAction) => {
            console.log("received", message);

            if (message.action === "graph/updateGraph") {
                data.graphManager.applyJson(message.payload);
            } else if (message.action === "registry/updateRegistry") {
                data.socketRegistry.applyJson(message.payload);
            } else if (message.action === "state/updateGlobalState") {
                data.globalEngineState.set(message.payload);
            }
        });
    }

    function setWindowDimensions(event: Event) {
        const target = event.target as Window;

        data.windowDimensions.set({
            width: target.innerWidth - 1,
            height: target.innerHeight - 3,
        });
    }

    function encodeResourceName(name: string) {
        return name.split("/").map(encodeURIComponent).join("/");
    }

    onMount(async () => {
        data.windowDimensions.set({
            width: window.innerWidth - 1,
            height: window.innerHeight - 3,
        });

        const resources = await getOrgan("organ");

        for (let resourcePath of resources) {
            if (Array.isArray(resourcePath)) {
                Promise.all(
                    resourcePath.map((subresource) =>
                        fetchSharedBuffer(
                            `/organ/${encodeResourceName(subresource)}`
                        )
                    )
                ).then(([resource, associatedResource]) => {
                    const toSend = {
                        type: "resource",
                        resource,
                        associatedResource,
                        path: resourcePath[0],
                    };

                    data.socket.sendRaw(toSend);
                });
            } else {
                fetch(`/organ/${resourcePath}`)
                    .then((data) => data.arrayBuffer())
                    .then((buffer) => new Uint8Array(buffer));
            }
        }
    });

    registerSocketEvents();
</script>

<svelte:window
    on:keydown={onWindowKeydown}
    on:resize={setWindowDimensions}
    on:click={onWindowClick}
/>

<FluentProvider bundles={[bundle]}>
    <Toasts />
    <slot />
</FluentProvider>

<style>
    :global(input) {
        height: 26px;
        border: none;
        outline: none;
        border-radius: 0;
        box-shadow: none;
        resize: none;
    }

    :global(input:focus-visible) {
        outline: 1px solid blue;
        border-radius: 0;
    }

    :global(select) {
        border: none;
        outline: none;
        border-radius: 0;
        box-shadow: none;
        resize: none;
    }

    :global(select:focus-visible) {
        outline: 1px solid blue;
        border-radius: 0;
    }
</style>
