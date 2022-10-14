<script lang="ts">
    import { SocketType, SocketDirection, type Primitive } from "../node-engine/connection";
    import { NodeRow, NodeWrapper, SocketValue } from "../node-engine/node";
    import { NodeGraph } from "../node-engine/node_graph";
    import { fixDigits } from "../util/fix-digits";
    import { BehaviorSubject, Observable } from "rxjs";

    import Socket from "./Socket.svelte";
    import { MemberType } from "safety-match";
    import { match, matchOrElse } from "../util/discriminated-union";

    export let type: SocketType;
    export let label: BehaviorSubject<string>;
    export let direction: SocketDirection;
    export let nodeWrapper: NodeWrapper;
    export let nodes: NodeGraph;

    let shouldDisplayDefaultField =
        direction === SocketDirection.Input
            ? nodes.getNodeInputConnection(nodeWrapper.index, type)
            : new Observable();

    let socketDefault: BehaviorSubject<SocketValue> = new BehaviorSubject({ variant: "None" });

    nodes.getNodeSocketDefault(nodeWrapper.index, type, direction).subscribe(socketDefault);

    function updateOverrides(event) {
        const newValue = event.target.value;

        const newValueParsed = matchOrElse(
            socketDefault.getValue(),
            {
                Stream: (): SocketValue => {
                    const num = parseFloat(newValue);
                    event.target.value = num;

                    return { variant: "Stream", data: isNaN(num) ? 0.0 : num };
                },
                Primitive: ({ data: primitiveType }): SocketValue => {
                    return {
                        variant: "Primitive",
                        data: match(primitiveType, {
                            String: (): Primitive => ({
                                variant: "String",
                                data: event.target.value,
                            }),
                            Int: (_) => {
                                const num = parseInt(newValue);
                                event.target.value = num;

                                return { variant: "Int", data: num };
                            },
                            Float: (_) => {
                                const num = parseFloat(newValue);
                                event.target.value = num;

                                return {
                                    variant: "Float",
                                    data: isNaN(num) ? 0.0 : num,
                                };
                            },
                            // booleans are special
                            Boolean: (_) => ({
                                variant: "Boolean",
                                data: event.target.checked,
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
        let override = nodeWrapper.default_overrides.find((defaultOverride) => {
            const { socketType: overrideSocketType, direction: overrideDirection } =
                NodeRow.getTypeAndDirection(defaultOverride);

            return (
                SocketType.areEqual(type, overrideSocketType) &&
                direction === overrideDirection
            );
        });

        if (override) {
            override.data[1] = (newValueParsed as any).data;
        } else {
            nodeWrapper.default_overrides = [
                ...nodeWrapper.default_overrides,
                NodeRow.fromTypeAndDirection(type, direction, newValueParsed),
            ];

            nodes.updateNode(nodeWrapper.index);
        }

        nodes.markNodeAsUpdated(nodeWrapper.index);
        nodes.writeChangedNodesToServer();
    }
</script>

<div
    class="container"
    class:output={direction === SocketDirection.Output}
    class:input={direction === SocketDirection.Input}
>
    {#if direction === SocketDirection.Input}
        <Socket {direction} {type} on:socketMousedown on:socketMouseup />
    {/if}

    {#if direction === SocketDirection.Input && !$shouldDisplayDefaultField}
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
                        <span class="input-hover-text">{$label}</span>
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
                    <span class="input-hover-text">{$label}</span>
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
    }

    label > span,
    label > input {
        position: absolute;
        margin: 0;
    }

    label > span {
        color: #777;
        margin-right: -80px;
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

    .input {
        text-align: left;
    }

    .output {
        text-align: right;
    }

    .text {
        display: inline-block;
        color: white;
    }
</style>
