import { writable, type Writable } from 'svelte/store';
import type { LayoutLoad } from './$types';

import { WasmIpcSocket, WebIpcSocket } from '$lib/ipc/socket';
import { GraphManager } from '$lib/node-engine/graph_manager';
import type { GlobalState, Resources } from '$lib/node-engine/global_state';

export const ssr = false;

export const load = (() => {
    // "ws://localhost:26642"
    let socket = new WebIpcSocket("ws://localhost:26642");
    const globalEngineState: Writable<GlobalState> = writable({
        activeProject: null,
        soundConfig: { sampleRate: 0 },
        resources: {
            ui: {}
        },
        ioRoutes: {
            rules: [],
            devices: []
        },
        devices: {
            midi: {},
            streams: {}
        }
    });
    const globalResources: Writable<Resources> = writable({
        ui: {}
    });

    return {
        socket,
        graphManager: new GraphManager(socket),
        globalEngineState,
        globalResources,
        windowDimensions: writable({ width: 400, height: 400 })
    };
}) satisfies LayoutLoad;