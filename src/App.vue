<template>

    <title-bar/>

    <div id="main">
        <router-view/>
    </div>

    <Loading/>

</template>

<style src="./styles/Themes.less" lang="less"/>
<style src="./styles/Main.less" lang="less"/>

<!--对 element-plus 的修改-->
<style src="./styles/Element.less" lang="less"/>

<script lang="ts" setup>

import {themeStore} from "./store/theme.ts";
import TitleBar from "./components/window/TitleBar.vue";
import hotkeys from 'hotkeys-js';
import {useRouter} from "vue-router";
import Loading from "./components/Loading.vue";

const theme = themeStore();
document.body.classList.add(theme.current);
const router = useRouter();


hotkeys('ctrl+s', (e) => e.preventDefault());
hotkeys('esc', (e) => {
    router.back();
});
hotkeys('f11, enter', fullScreen);

let full = false;

function fullScreen() {

    full = !full;

    if (full) {
        document.body.requestFullscreen();
    } else {
        document.exitFullscreen();
    }
}

</script>