<script lang="ts">
	import PropertyEditor from './node-editor/PropertyEditor.svelte';
	import Editor from './node-editor/Editor.svelte';
	import SideNavbar from './node-editor/SideNavbar.svelte';
	import SplitView from './layout/SplitView.svelte';
	import {SplitDirection} from './layout/enums';
	import {windowDimensions} from './util/window-size';
	
	const ipc = (window as any).ipcRenderer;

	function sendJson (json) {
		ipc.send("send", json);
	}

	// sendJson({
	// 	"foo": "bar"
	// });

	ipc.on("receive", function(event: object, message: object) {
		console.log(message);
	});

	let width = 0;
	let height = 0;

	windowDimensions.subscribe(([windowWidth, windowHeight]) => {
		width = windowWidth - 1;
		height = windowHeight - 3;
	});
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
	secondPanel={SplitView}
	secondState={{
		direction: SplitDirection.VERTICAL,
		firstPanel: PropertyEditor,
		secondPanel: Editor
	}} />
</main>

<style>
</style>