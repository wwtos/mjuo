<script lang="ts">
    import { FluentBundle, FluentResource } from "@fluent/bundle";
    import { FluentProvider } from "@nubolab-ffwd/svelte-fluent";

    import enTranslations from "$lib/assets/lang/en.ftl?raw";
    import SideNavbar from "./node_editor/SideNavbar.svelte";
    import Toasts from "$lib/components/Toasts.svelte";
    import type { PageData } from "./$types";
    import type { IpcAction } from "$lib/ipc/action";
    import { setContext } from "svelte";
    import { constructEngine } from "./engine";
    import type { WasmIpcSocket } from "$lib/ipc/socket";

    export let data: PageData;

    const bundle = new FluentBundle("en");
    bundle.addResource(new FluentResource(enTranslations));

    constructEngine().then((engine) => {
        console.log(engine);
        (data.socket as WasmIpcSocket).setEngine(engine);
    });

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

    function onWindowResize(this: Window) {
        data.windowDimensions.set({
            width: this.innerWidth - 1,
            height: this.innerHeight - 3,
        });
    }

    function registerSocketEvents() {
        data.socket.onMessage((message: IpcAction) => {
            console.log("received", data);

            if (message.action === "graph/updateGraph") {
                data.graphManager.applyJson(message.payload);
            } else if (message.action === "registry/updateRegistry") {
                data.socketRegistry.applyJson(message.payload);
            } else if (message.action === "state/updateGlobalState") {
                data.globalEngineState.set(message.payload);
            }
        });
    }

    registerSocketEvents();
</script>

<svelte:window on:keydown={onWindowKeydown} on:resize={onWindowResize} />

<FluentProvider bundles={[bundle]}>
    <Toasts />
    <div style="display: flex">
        <SideNavbar />
        <slot />
    </div>
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
