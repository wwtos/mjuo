<script lang="ts">
    import { IPCSocket } from "../util/socket";
    import { SvelteToast, toast } from '@zerodevx/svelte-toast';

    export let ipcSocket: IPCSocket;

    ipcSocket.onMessage(message => {
        if(message.action === "toast/error") {
            toast.push(message.payload, {
                theme: {
                    '--toastBackground': '#F56565',
                    '--toastBarBackground': '#C53030'
                }
            });
        }
    });
</script>

<SvelteToast />
