<script lang="ts">
	import Editor from './node-editor/Editor.svelte';
	import SideNavbar from './node-editor/SideNavbar.svelte';
	import SplitView from './layout/SplitView.svelte';
	import {SplitDirection} from './layout/enums';
	import {windowDimensions} from './util/window-size';
	import {IPCSocket} from './util/socket';
	import {NodeGraph} from './node-engine/node_graph';
	import Toasts from './ui/Toasts.svelte';
	import { graphManager, ipcSocket, socketRegistry } from './node-editor/state';
	import { BehaviorSubject } from 'rxjs';
	
	const ipc = (window as any).ipcRenderer;
	let newIpcSocket: any = new IPCSocket(ipc);

	ipcSocket.next(newIpcSocket);
	graphManager.setIpcSocket(newIpcSocket);

	window["graphManager"] = graphManager;

	newIpcSocket.onMessage(([message]) => {
        console.log("received", message);

        if (message.action === "graph/updateGraph") {
            graphManager.applyJson(message);
        } else if (message.action === "registry/updateRegistry") {
            $socketRegistry.applyJson(message.payload);
        }
    });

	let width = 0;
	let height = 0;

	windowDimensions.subscribe(([windowWidth, windowHeight]) => {
		width = windowWidth - 1;
		height = windowHeight - 3;
	});

	let activeGraph = new BehaviorSubject<NodeGraph>(graphManager.getRootGraph());
</script>

<main>
	<!-- <div id="main-flex">
		<SideNavbar />
		<Editor />
	</div> -->
	<SplitView 
	direction={SplitDirection.VERTICAL}
	{width} {height}
	hasFixedWidth={true} fixedWidth={48}
	firstPanel={SideNavbar}
	secondPanel={Editor}
	secondState={{
		ipcSocket: newIpcSocket,
		activeGraph: activeGraph
	}} />
	<Toasts ipcSocket={newIpcSocket} />
	<!-- <SplitView 
	direction={SplitDirection.VERTICAL}
	{width} {height}
	hasFixedWidth={true} fixedWidth={48}
	firstPanel={SideNavbar}
	secondPanel={Editor}
	secondState={{
		ipcSocket: ipcSocket,
		nodes: nodes
	}} /> -->
</main>

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