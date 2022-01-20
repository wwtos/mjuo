<script>
import NumberProperty from "./NumberProperty.svelte";
import OptionsProperty from "./OptionsProperty.svelte";

export let width;
export let height;
</script>

<div style="width: {width}px; height: {height}px">
    <div class="container">
        <div class="row">
            <div class="prop-name">Number</div>
            <div class="dividing-bar"></div>
            <div class="prop-value"><NumberProperty value={0} /></div>
        </div>
        <div class="row">
            <div class="prop-name">Stuff</div>
            <div class="dividing-bar"></div>
            <div class="prop-value"><OptionsProperty states={{foo: "bar", baz: "qux"}} value={"baz"} /></div>
        </div>
    </div>
</div>

<style>    
.dividing-bar {
    flex-grow: 0;
    border-left: 1px solid rgb(216, 216, 216);
}

.container {
    display: flex;
    flex-direction: column;
}

.row {
    width: 100%;
    display: inline-flex;
    flex-direction: row;
    justify-content: center;
    padding: 0;
    border-bottom: 1px solid rgb(216, 216, 216);
}

.row:hover {
    background-color: rgb(223, 223, 223);
}

.prop-name, .prop-value {
    width: 50%;
    margin: 0;
}

.prop-name {
    text-align: right;
    padding: 3px 6px 3px 5px;
}

.prop-value {
    padding: 0;
}
</style>