<script lang="ts">
    import type { Index } from "$lib/ddgg/gen_vec";
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import { match, matchOrElse } from "$lib/util/discriminated-union";
    import { fixDigits } from "$lib/util/fix-digits";
    import { deepEqual } from "fast-equals";

    import type {
        SocketDirection,
        Socket,
        SocketValue,
        Primitive,
    } from "$lib/node-engine/connection";
    import UiSocket from "./UiSocket.svelte";
    import { createEventDispatcher } from "svelte";

    export let socket: Socket;
    export let label: string;
    export let direction: SocketDirection;
    export let nodeIndex: Index;
    export let nodes: NodeGraph;
    export let value: SocketValue;

    const dispatch = createEventDispatcher();

    $: connectedFrom =
        direction.variant === "Input"
            ? nodes.getNodeInputConnection(nodeIndex, socket)
            : undefined;

    function bubbleOverrides(this: HTMLInputElement, event: Event) {
        const newValueRaw = this.value;

        const newValueParsed = matchOrElse(
            value,
            {
                Stream: (): SocketValue => {
                    const num = parseFloat(newValueRaw);
                    this.value = num + "";

                    return { variant: "Stream", data: isNaN(num) ? 0.0 : num };
                },
                Value: ({ data: primitiveType }): SocketValue => {
                    return {
                        variant: "Value",
                        data: match(primitiveType, {
                            String: (): Primitive => ({
                                variant: "String",
                                data: this.value,
                            }),
                            Int: (_): Primitive => {
                                const num = parseInt(newValueRaw);
                                this.value = num + "";

                                return { variant: "Int", data: num };
                            },
                            Float: (_): Primitive => {
                                const num = parseFloat(newValueRaw);
                                this.value = num + "";

                                return {
                                    variant: "Float",
                                    data: isNaN(num) ? 0.0 : num,
                                };
                            },
                            // booleans are special
                            Boolean: (_): Primitive => ({
                                variant: "Boolean",
                                data: this.checked,
                            }),
                        }),
                    };
                },
            },
            () => {
                throw new Error("unimplemented");
            }
        );

        dispatch("overrideUpdate", {
            socket,
            direction,
            newValue: newValueParsed,
        });
    }
</script>

<div
    class="container"
    class:socket-output={direction?.variant === "Output"}
    class:socket-input={direction?.variant === "Input"}
>
    {#if direction.variant === "Input"}
        <UiSocket {direction} {socket} on:socketMousedown on:socketMouseup />
    {/if}

    {#if direction.variant === "Input" && !connectedFrom}
        {#if value.variant === "Value"}
            {#if value.data.variant === "Float"}
                <div class="flex">
                    <label>
                        <input
                            value={fixDigits(value.data.data, 3)}
                            on:mousedown|stopPropagation
                            on:change={bubbleOverrides}
                            on:keydown|stopPropagation
                        />
                        <div>
                            <span class="input-hover-text">{label}</span>
                        </div>
                    </label>
                </div>
            {:else if value.data.variant === "Boolean"}
                <input
                    type="checkbox"
                    on:change={bubbleOverrides}
                    on:mousedown|stopPropagation
                    on:dblclick|stopPropagation
                    checked={value.data.data}
                />
                <div class="text">{label}</div>
            {/if}
        {:else if value.variant === "Stream"}
            <div class="flex">
                <label>
                    <input
                        value={fixDigits(value.data, 3)}
                        on:mousedown|stopPropagation
                        on:change={bubbleOverrides}
                        on:keydown|stopPropagation
                    />
                    <div>
                        <span class="input-hover-text">{label}</span>
                    </div>
                </label>
            </div>
        {:else}
            <div class="text">{label}</div>
        {/if}
    {:else}
        <div class="text">{label}</div>
    {/if}

    {#if direction.variant === "Output"}
        <UiSocket {direction} {socket} on:socketMousedown on:socketMouseup />
    {/if}
</div>

<style>
    label {
        position: relative;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: text;
        width: calc(100% - 40px);
    }

    label > div {
        position: absolute;
        width: 100%;
        display: flex;
        justify-content: flex-end;
        flex-direction: row;
    }

    label > input {
        position: absolute;
        margin: 0;
        width: 100%;
    }

    label > div > span {
        color: #777;
        margin: 0 12px;
    }

    .flex {
        display: flex;
        flex-flow: column;
        align-items: center;
        margin-top: -12px;
    }

    input {
        height: 26px;
        border: none;
        border-radius: 5px;
        outline: none;
        box-shadow: none;
        resize: none;
    }

    input:focus-visible {
        outline: 1px solid blue;
    }

    input[type="checkbox"] {
        height: initial;
    }

    .container {
        margin: 10px 0;
        height: 26px;
    }

    .socket-input {
        text-align: left;
    }

    .socket-output {
        text-align: right;
    }

    .text {
        display: inline-block;
        color: white;
    }
</style>
