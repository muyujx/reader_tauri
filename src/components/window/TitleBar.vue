<template>

    <div class="title-bar">

        <div class="menu">
        </div>

        <div class="buttons">
            <div id="minimize" @click="minimize">-</div>
            <div id="maximize" @click="maximize">□</div>
            <div id="close" @click="close">×</div>
        </div>

    </div>


</template>

<script setup lang="ts">

// 最小化
import {ipcSend} from "../../utils/ipcUtil.ts";

function minimize() {
    ipcSend('window_minimize');
}

// 最大化
function maximize() {
    ipcSend('window_maximize');
}

// 关闭程序
function close() {
    ipcSend('window_close');
}


</script>

<style scoped lang="less">

.title-bar {
    width: 100%;
    height: 30px;

    // 配置 electron 窗口上面可以用来拖动
    app-region: drag;

    background-color: var(--background);
    display: flex;
    flex-direction: row;
    justify-content: space-between;

    * {
        user-select: none;
    }
}

.menu {
    display: flex;
    flex-direction: column;
    padding-left: 20px;
    app-region: no-drag;
    color: var(--foreground);
}

.buttons {
    display: flex;
    align-items: center;
    justify-content: end;
    app-region: no-drag;

    color: var(--foreground);

    & div {
        height: 100%;
        width: 45px;
        font-size: 20px;

        display: flex;
        justify-content: center;
        align-items: center;

        &:hover {
            color: #FFFFFF;
            background-color: rgba(255, 255, 255, 0.4);
        }
    }

    #close:hover {
        background-color: rgba(196, 43, 28, 0.7);
    }


}


</style>