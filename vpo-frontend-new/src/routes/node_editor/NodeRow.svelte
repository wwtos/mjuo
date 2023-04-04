<script lang="ts">
    import type { Index } from "$lib/ddgg/gen_vec";
    import {
        SocketDirection,
        type Primitive,
        SocketType,
    } from "$lib/node-engine/connection";
    import {
        type NodeWrapper,
        type SocketValue,
        NodeRow,
    } from "$lib/node-engine/node";
    import type { NodeGraph } from "$lib/node-engine/node_graph";
    import { match, matchOrElse } from "$lib/util/discriminated-union";
    import { fixDigits } from "$lib/util/fix-digits";
    import { deepEqual } from "fast-equals";
    import { BehaviorSubject, Observable } from "rxjs";

    import Socket from "./Socket.svelte";

    export let type: SocketType;
    export let label: BehaviorSubject<string>;
    export let direction: SocketDirection;
    export let polyphonic: boolean;
    export let nodeWrapper: NodeWrapper;
    export let nodeIndex: Index;
    export let nodes: NodeGraph;

    let shouldNotDisplayDefaultField =
        direction === SocketDirection.Input
            ? nodes.getNodeInputConnection(nodeIndex, type)
            : new Observable();

    const none: SocketValue = {
        variant: "None",
    };

    let socketDefault: BehaviorSubject<SocketValue> = new BehaviorSubject(none);

    nodes
        .getNodeSocketDefault(nodeIndex, type, direction)
        .subscribe(socketDefault);

    function updateOverrides(this: HTMLInputElement, event: Event) {
        const newValue = this.value;

        const newValueParsed = matchOrElse(
            socketDefault.getValue(),
            {
                Stream: (): SocketValue => {
                    const num = parseFloat(newValue);
                    this.value = num + "";

                    return { variant: "Stream", data: isNaN(num) ? 0.0 : num };
                },
                Primitive: ({ data: primitiveType }): SocketValue => {
                    return {
                        variant: "Primitive",
                        data: match(primitiveType, {
                            String: (): Primitive => ({
                                variant: "String",
                                data: this.value,
                            }),
                            Int: (_) => {
                                const num = parseInt(newValue);
                                this.value = num + "";

                                return { variant: "Int", data: num };
                            },
                            Float: (_) => {
                                const num = parseFloat(newValue);
                                this.value = num + "";

                                return {
                                    variant: "Float",
                                    data: isNaN(num) ? 0.0 : num,
                                };
                            },
                            // booleans are special
                            Boolean: (_) => ({
                                variant: "Boolean",
                                data: this.checked,
                            }),
                        }),
                    };
                },
            },
            () => {
                throw "unimplemented";
            }
        );

        // check if this override is already in there, in which case the value needs to be updated
        let override = nodeWrapper.defaultOverrides.find((defaultOverride) => {
            const {
                socketType: overrideSocketType,
                direction: overrideDirection,
            } = NodeRow.getTypeAndDirection(defaultOverride) ?? {};

            return (
                deepEqual(type, overrideSocketType) &&
                direction === overrideDirection
            );
        });

        if (override && override.data) {
            override.data[1] = (newValueParsed as any).data;
        } else {
            nodeWrapper.defaultOverrides = [
                ...nodeWrapper.defaultOverrides,
                NodeRow.fromTypeAndDirection(
                    type,
                    direction,
                    (newValueParsed as any).data,
                    polyphonic
                ),
            ];

            nodes.updateNode(nodeIndex);
        }

        nodes.markNodeAsUpdated(nodeIndex);
        nodes.writeChangedNodesToServer();
    }
</script>

<div
    class="container"
    class:socket-output={direction === SocketDirection.Output}
    class:socket-input={direction === SocketDirection.Input}
>
    {#if direction === SocketDirection.Input}
        <Socket {direction} {type} on:socketMousedown on:socketMouseup />
    {/if}

    {#if direction === SocketDirection.Input && !$shouldNotDisplayDefaultField}
        {#if $socketDefault.variant === "Primitive"}
            {#if $socketDefault.data.variant === "Float"}
                <div class="flex">
                    <label>
                        <input
                            value={fixDigits($socketDefault.data.data, 3)}
                            on:mousedown={(e) => e.stopPropagation()}
                            on:change={updateOverrides}
                            on:keydown={(event) => event.stopPropagation()}
                        />
                        <div>
                            <span class="input-hover-text">{$label}</span>
                        </div>
                    </label>
                </div>
            {:else if $socketDefault.data.variant === "Boolean"}
                <input
                    type="checkbox"
                    on:change={updateOverrides}
                    on:mousedown={(e) => e.stopPropagation()}
                    checked={$socketDefault.data.data}
                />
                <div class="text">{$label}</div>
            {/if}
        {:else if $socketDefault.variant === "Stream"}
            <div class="flex">
                <label>
                    <input
                        value={fixDigits($socketDefault.data, 3)}
                        on:mousedown={(e) => e.stopPropagation()}
                        on:change={updateOverrides}
                        on:keydown={(event) => event.stopPropagation()}
                    />
                    <div>
                        <span class="input-hover-text">{$label}</span>
                    </div>
                </label>
            </div>
        {:else}
            <div class="text">{$label}</div>
        {/if}
    {:else}
        <div class="text">{$label}</div>
    {/if}

    {#if direction === SocketDirection.Output}
        <Socket {direction} {type} on:socketMousedown on:socketMouseup />
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
        border-radius: 5px;
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
