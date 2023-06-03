<script lang="ts">
    import { FluentBundle, FluentResource } from "@fluent/bundle";
    import { FluentProvider } from "@nubolab-ffwd/svelte-fluent";

    import enTranslations from "$lib/assets/lang/en.ftl?raw";
    import SideNavbar from "./node_editor/SideNavbar.svelte";
    import { toast, SvelteToast } from "@zerodevx/svelte-toast";
    import type { PageData } from "./$types";
    import type { IpcAction } from "$lib/ipc/action";
    import { constructEngine } from "./engine";
    import { onMount } from "svelte";
    import { parse } from "toml";

    import { fetchSharedBuffer } from "$lib/util/fetch-shared-buffer";

    export let data: PageData;

    const bundle = new FluentBundle("en");
    bundle.addResource(new FluentResource(enTranslations));

    // const context = new AudioContext({
    //     sampleRate: 48000,
    //     latencyHint: "interactive",
    // });

    // constructEngine(context).then((engine) => {
    //     data.socket.setEngine(engine);
    // });

    // function getOrgan(organ: string) {
    //     return fetch(`/${organ}/resources.json`).then((data) => data.json());
    // }

    function onWindowKeydown(event: KeyboardEvent) {
        if (event.ctrlKey) {
            switch (event.key) {
                case "s":
                    data.socket.save();
                    event.preventDefault();
                    break;
                case "o":
                    data.socket.load();
                    event.preventDefault();
                    break;
            }
        }
    }

    async function onWindowClick() {
        // if (context.state !== "running") {
        //     await context.resume();
        //     data.socket.flushMessages();
        // }
    }

    function registerSocketEvents() {
        data.socket.onMessage((message: IpcAction) => {
            console.log("received", message);

            if (message.action === "graph/updateGraph") {
                data.graphManager.applyJson(message.payload);
            } else if (message.action === "registry/updateRegistry") {
                data.socketRegistry.applyJson(message.payload);
            } else if (message.action === "state/updateGlobalState") {
                const newGlobalState = message.payload;

                for (let i in newGlobalState.resources.ui) {
                    newGlobalState.resources.ui[i] = parse(
                        newGlobalState.resources.ui[i]
                    );
                }

                data.globalEngineState.set(newGlobalState);
            } else if (message.action === "toast/error") {
                toast.push({
                    msg: message.payload,
                });
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

        // const resources = await getOrgan("organ");

        // for (let resourcePath of resources) {
        //     if (Array.isArray(resourcePath)) {
        //         Promise.all(
        //             resourcePath.map((subresource) =>
        //                 fetchSharedBuffer(
        //                     context,
        //                     `/organ/${encodeResourceName(subresource)}`
        //                 )
        //             )
        //         ).then(([resource, associatedResource]) => {
        //             const toSend = {
        //                 type: "resource",
        //                 resource,
        //                 associatedResource,
        //                 path: resourcePath[0],
        //             };

        //             data.socket.sendRaw(toSend);
        //         });
        //     } else {
        //         fetchSharedBuffer(context, `/organ/${resourcePath}`).then(
        //             (resource) => {
        //                 const toSend = {
        //                     type: "resource",
        //                     resource,
        //                     path: encodeResourceName(resourcePath),
        //                 };

        //                 data.socket.sendRaw(toSend);
        //             }
        //         );
        //     }
        // }
    });

    registerSocketEvents();
</script>

<svelte:window
    on:keydown={onWindowKeydown}
    on:resize={setWindowDimensions}
    on:click={onWindowClick}
/>

<FluentProvider bundles={[bundle]}>
    <SvelteToast />
    <div class="main">
        <slot />
    </div>
</FluentProvider>

<style>
    .main {
        overflow: hidden;
    }
</style>
