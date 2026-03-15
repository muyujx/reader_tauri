<template>

    <div class="config">

        <div class="config-content">


            <div class="config-item">

                <div class="config-title">
                    当前账号:
                </div>

                <div class="account">
                    <p>{{ userInfo.account }}</p>
                    <div class="button" @click="logout">登出</div>
                </div>
            </div>


            <div class="config-item hide-on-mobile">

                <div class="config-title">
                    初始窗口:
                </div>

                <div class="start-win-size">
                    <p>{{ winSize[0] }} * {{ winSize[1] }}</p>
                    <div class="button" @click="setStartWinSize">设置为当前窗口大小</div>
                </div>
            </div>


            <div class="config-item">

                <div class="config-title">
                    缓存目录:
                </div>

                <div class="cache-dir" @click="changeCacheDir">
                    {{ cacheDir }}
                </div>
            </div>

            <div class="config-item">

                <div class="config-title">
                    主题选择:
                </div>

                <div class="theme-list">
                    <div class="theme-item"
                         v-for="(value, name) in Theme"
                         :class="{
                             [value]: true,
                             'active': value === curTheme.current
                         }"
                         :key="value"
                         @click.stop="chooseTheme(value)">
                        <p>{{ name }}</p>
                    </div>
                </div>

            </div>

            <div class="config-item scale-config">
                <div class="config-title">
                    书页放大:
                </div>

                <el-slider
                    class="scale-select"
                    v-model="scale.current"
                    @change="scaleChange"
                    :show-tooltip="false"
                    :min="0"
                    :max="50"/>

                <div class="scale-indicator">
                    {{ scale.current }}
                </div>
            </div>

            <div class="config-item hide-on-mobile">

                <div class="config-title">
                    快捷键:
                </div>

                <div class="short-cut-key">
                    <div>返回: <kbd>ESC</kbd></div>
                    <div>向前翻页: <kbd>A</kbd> <kbd>S</kbd> <kbd>&#x2190;</kbd></div>
                    <div>向后翻页: <kbd>D</kbd> <kbd>F</kbd> <kbd>&#x2192;</kbd></div>
                    <div>开关目录: <kbd>Tab</kbd></div>
                    <div>全屏开关: <kbd>F11</kbd> <kbd>Enter</kbd></div>
                    <div>快速选择书页中的图片: <kbd>W</kbd></div>
                </div>
            </div>

            <div class="config-item">
                <div class="config-title">
                    版本:
                </div>

                <div class="config-text">{{ version }}</div>
            </div>

            <div class="config-item">
                <div class="config-title">
                    构建时间:
                </div>

                <div class="config-text">{{ buildTime }}</div>
            </div>


        </div>


    </div>


</template>

<script setup lang="ts">
import {ref} from "vue";
import {Theme, themeStore} from "../../store/theme";
import {setLocalStorage} from "../../utils/localStorageUtil";
import {PageType} from "../../store/pageType";
import {scaleStore} from "../../store/scale";
import {ipcInvoke} from "../../utils/ipcUtil.ts";
import {open} from "@tauri-apps/plugin-dialog";
import ipcChannel from "../../common/ipcChannel.ts";
import {userStore} from "../../store/userStore.ts";
import {logoutApi} from "../../apis/login.ts";
import {useRouter} from "vue-router";


// const selector = ref();
const curTheme = themeStore();
const scale = scaleStore();
const winSize = ref([0, 0]);
const userInfo = userStore();

const cacheDir = ref('');
const router = useRouter();

const version = import.meta.env.VITE_APP_VERSION;
const buildTime = import.meta.env.VITE_APP_BUILD_TIME;

async function init() {
    // 获取当前的缓存根目录
    cacheDir.value = await ipcInvoke(ipcChannel.get_root_cache_dir);

    winSize.value = await ipcInvoke(ipcChannel.get_start_win_size);
}

init();

/**
 * 选择缓存文件夹
 */
async function changeCacheDir() {
    // 使用 Tauri dialog 插件直接选择文件夹
    const folder = await open({
        directory: true,
        title: "选择缓存目录"
    });
    
    if (folder == null) {
        return;
    }

    await ipcInvoke(ipcChannel.change_root_cache_dir, { dir: folder });
    cacheDir.value = await ipcInvoke(ipcChannel.get_root_cache_dir);
}


function chooseTheme(theme: Theme) {
    const bodyClassList = document.body.classList;
    bodyClassList.remove(curTheme.current);
    curTheme.current = theme;
    bodyClassList.add(theme);
    setLocalStorage("theme", theme);
}

function scaleChange(val: number) {
    setLocalStorage("scale", val);
}

async function setStartWinSize() {
    let width = window.outerWidth;
    let height = window.outerHeight;
    winSize.value = await ipcInvoke(ipcChannel.set_start_win_size, { width, height });
}

function logout() {
    logoutApi();
    router.push({
        name: "Login"
    });
}

</script>

<style scoped lang="less" src="./Config.less"/>