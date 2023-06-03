<script lang="ts">
    import { FILE_PREFIX } from "$lib/constants";

    type Layer =
        | {
              type: "image";
              image: string;
          }
        | {
              type: "text";
              template: string;
              offset?: [number, number];
              style?: string;
          };

    export let layer: Layer;
</script>

{#if layer.type === "image"}
    <img
        src={FILE_PREFIX + layer.image.replace(":", "/")}
        draggable="false"
        alt=""
        style="position: absolute; left: 0px; top: 0px"
    />
{/if}
