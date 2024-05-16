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
    export let properties: { [key: string]: string };
    export let layerIndex: number;

    let layerStyle = "";

    $: {
        if (layer.type === "text") {
            layerStyle = layer.style?.trim() || "";

            if (layerStyle.lastIndexOf(";") !== layerStyle.length - 1) {
                layerStyle += ";";
            }

            layerStyle += `z-index: ${layerIndex};`;

            if (layer.offset) {
                layerStyle += `left: ${layer.offset[0]}px; top: ${layer.offset[1]}px`;
            }
        }
    }

    // https://stackoverflow.com/questions/3446170/escape-string-for-use-in-javascript-regex
    function escapeRegExp(string: string) {
        return string.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    }

    function escapeReplacement(string: string) {
        return string.replace(/\$/g, "$$$$");
    }

    let text = "";

    $: if (layer.type === "text") {
        let replaced = layer.template;

        for (let prop in properties) {
            replaced = replaced.replace(
                new RegExp(escapeRegExp(`{${prop}}`)),
                escapeReplacement(properties[prop]),
            );
        }

        text = replaced;
    }
</script>

{#if layer.type === "image"}
    <img
        src={FILE_PREFIX + layer.image.replace(":", "/")}
        draggable="false"
        alt=""
        style="position: absolute; left: 0px; top: 0px; z-index: {layerIndex}"
    />
{:else if layer.type === "text"}
    <span style={layerStyle} class="text">{text}</span>
{/if}

<style>
    .text {
        position: absolute;
    }
</style>
