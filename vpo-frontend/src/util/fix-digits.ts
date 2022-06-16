export function fixDigits(num: number, digits: number) {
    const decimalParts = (Math.round(num * Math.pow(10, digits)) / Math.pow(10, digits)).toString().split(".");

    if (decimalParts.length > 1) {
        return decimalParts[0] + "." + decimalParts[1].substring(0, digits);
    } else {
        return decimalParts[0];
    }
}
