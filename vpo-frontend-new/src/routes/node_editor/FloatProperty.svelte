<script>
    export let onchange = function() {};
    export let value;
    export let step = 1;

    let displayedValue = value;
    
    let firstTime = true;
    
    $: {
        if(!firstTime && value !== undefined) {
            // when the input box value changes, parse it and set the output value to that
            value = parseFloat(displayedValue);
            onchange(value);
        }
    
        firstTime = false;
    }

    $: {
        const decimalParts = (Math.round(displayedValue / step) * step).toString().split(".");

        if (decimalParts.length > 1) {
            // TODO: this 2 should not be hardcoded
            displayedValue = decimalParts[0] + "." + decimalParts[1].substring(0, 2);
        } else {
            displayedValue = decimalParts[0];
        }
    }
    </script>
    
    <input type="number" bind:value={displayedValue} step={step} />
    
    <style>
    input {
        width: 100%;
        margin: 0;
    }
    </style>