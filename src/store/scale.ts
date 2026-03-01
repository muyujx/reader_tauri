import {defineStore} from "pinia";
import {getLocalStorageInt} from "../utils/localStorageUtil";

export const scaleStore = defineStore("scale", () => {
    const current = getLocalStorageInt("scale", 10);
    return {current};
});
