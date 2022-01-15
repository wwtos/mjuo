<script lang="ts">
	import Editor from './node-editor/Editor.svelte';
	import SideNavbar from './node-editor/SideNavbar.svelte';

	export let name: string;
</script>

<main>
	<div id="main-flex">
		<SideNavbar />
		<Editor />
	</div>
</main>

<style>
	#main-flex {
		display: flex;
		flex-direction: row;
		align-items: stretch;
	}
</style>