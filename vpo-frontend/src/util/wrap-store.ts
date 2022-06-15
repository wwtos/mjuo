import { Observable } from "rxjs";
import { Readable } from "svelte/store";

export function wrapStore<Type>(store: Readable<Type>): Observable<Type> {
    return new Observable(subscriber => {
        store.subscribe(newData => subscriber.next(newData));
    });
}
