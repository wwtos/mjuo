import { writable, type Writable } from 'svelte/store';
import type { LayoutLoad } from './$types';

import { WasmIpcSocket, WebIpcSocket } from '$lib/ipc/socket';
import { GraphManager } from '$lib/node-engine/graph_manager';
import { SocketRegistry } from '$lib/node-engine/socket_registry';
import type { GlobalState } from '$lib/node-engine/global_state';

export const ssr = false;

export const load = (() => {
    // "ws://localhost:26642"
    let socket = new WasmIpcSocket();
    const globalEngineState: Writable<GlobalState> = writable({
        activeProject: null,
        soundConfig: {sampleRate: 0},
        resources: []
    });

    return {
        socket,
        graphManager: new GraphManager(socket),
        socketRegistry: new SocketRegistry(),
        globalEngineState,
        windowDimensions: writable({width: 400, height: 400})
    };
}) satisfies LayoutLoad;