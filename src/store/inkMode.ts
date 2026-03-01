import {defineStore} from "pinia";
import {getLocalStorageBoolean} from "../utils/localStorageUtil";

export const inkModeStore = defineStore("inkMode", () => {
    const current = getLocalStorageBoolean("inkMode", false);
    return {current};
});
