import type { Socket } from "$lib/node-engine/connection";
import { derived, type Readable } from "svelte/store";

type LocalizeFn = (id: string, args?: Record<string, string> | undefined) => string;
function localizeSocket(localize: LocalizeFn, socket: Socket): string {
    return localize("socket." + socket.data[0], socket.variant === "WithData" ? { x: socket.data[1] } : undefined);
}

export { localizeSocket };