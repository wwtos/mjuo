import { writable } from 'svelte/store';

export const windowDimensions = writable([window.innerWidth, window.innerHeight]);

window.addEventListener("resize", () => {
    windowDimensions.set([window.innerWidth, window.innerHeight]);
});
