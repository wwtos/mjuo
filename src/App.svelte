<script lang="ts">
	import Editor from './node-editor/Editor.svelte';
	import SideNavbar from './node-editor/SideNavbar.svelte';
	import SplitView from './layout/SplitView.svelte';
	import {SplitDirection} from './layout/enums';
	import {windowDimensions} from './util/window-size';

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
	firstPanel={Editor}
	secondPanel={Editor} />
</main>

<style>
	#main-flex {
		display: flex;
		flex-direction: row;
		align-items: stretch;
	}
</style>