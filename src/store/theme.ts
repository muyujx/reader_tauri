import {defineStore} from "pinia";
import {getLocalStorage} from "../utils/localStorageUtil";

export enum Theme {

    // dark

    Oceanic = 'theme-oceanic',

    Darker = 'theme-darker',

    PaleNight = 'theme-palenight',

    DeepOcean = 'theme-deepocean',

    Monokai = 'theme-monokai',

    Dracula = 'theme-dracula',

    GithubDark = 'theme-githubdark',

    ArcDark = 'theme-arcdark',

    OneDark = 'theme-onedark',

    NightOwl = 'theme-nightowl',

    Moonlight = 'theme-moonlight',

    SynthWave = 'theme-synthwave',

    SolarDark = 'theme-solardark',

    Forest = 'theme-forest',

    Volcano = 'theme-volcano',


    // light

    SandyBeach = 'theme-sandybeach',

    Lighter = 'theme-lighter',

    Github = 'theme-github',

    LightOwl = 'theme-lightowl',

    Skyblue = 'theme-skyblue',

    OneLight = 'theme-onelight',

    // 墨水屏幕
    InkScreen = "theme-ink",

}

const defaultTheme = Theme.SolarDark;

export const themeStore = defineStore("theme", () => {
    const current = getLocalStorage("theme", defaultTheme);
    return {current};
});
