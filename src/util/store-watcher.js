// TODO: verify this doesn't leak memory
export function storeWatcher(store) {
    this.store = store;
    this.value;

    this.store.subscribe(val => {
        this.value = val;
    })
}

storeWatcher.prototype.get = function() {
    return this.value;
};
