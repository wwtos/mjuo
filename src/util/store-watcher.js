// TODO: verify this doesn't leak memory
export class storeWatcher {
    constructor (store) {
        this.store = store;

        this.store.subscribe(val => {
            this.value = val;
        });
    }    

    get () {
        return this.value;
    }
}
