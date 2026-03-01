import {defineStore} from "pinia";
import {getLocalStorageInt} from "../utils/localStorageUtil";


const DEFAULT_FONT_SIZE = 18;

export const fontSizeStore = defineStore("fontSize", () => {
    const current = getLocalStorageInt("fontSize", DEFAULT_FONT_SIZE);
    return {current};
});
