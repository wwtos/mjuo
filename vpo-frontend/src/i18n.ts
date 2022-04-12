import i18next from "i18next";
import { createI18nStore } from "svelte-i18next";
import en from "./lang/en.json";

i18next.init({
    lng: 'en',
    resources: {
        en
    },
    interpolation: {
       escapeValue: false, // not needed for svelte as it escapes by default
    }
});

window["i18nInstance"] = en;

export const i18nStore = createI18nStore(i18next);
export const i18n = i18next;