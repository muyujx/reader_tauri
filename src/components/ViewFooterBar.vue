<template>

    <div class="footer_bar" ref="footerBar">

        <div @click="openContents">
            <el-icon>
                <Operation/>
            </el-icon>
            <p>目录</p>
        </div>

        <div @click="toBookList">
            <el-icon>
                <Menu/>
            </el-icon>
            <p>书架</p>
        </div>


        <div @click="openConfig">
            <el-icon>
                <Sunny/>
            </el-icon>
            <p>设置</p>
        </div>

    </div>

</template>

<script setup lang="ts">
import {Operation, Menu, Sunny} from "@element-plus/icons-vue";
import {useRouter} from "vue-router";
import {ref} from "vue";

const emit = defineEmits<{
    (e: 'openContents'): void,
    (e: 'openConfig'): void,
}>();


const router = useRouter();
const footerBar = ref(null);

function toBookList() {
    router.go(-1);
}

function openContents() {
    emit('openContents');
}

function openConfig () {
    emit("openConfig");
}

</script>


<style scoped lang="less">

.footer_bar {
    display: none;
}

.footer_bar {

    background-color: var(--background);
    width: 100%;
    height: 70px;
    position: fixed;
    bottom: 0;
    left: 0;
    z-index: 10;

    div {
        //background-color: #00BCD4;
        width: 60px;
        height: 60px;
        color: var(--text);

        display: flex;
        flex-direction: column;
        align-items: center;

        .el-icon {
            width: 30px;
            height: 30px;
            display: block;
            margin-left: auto;
            margin-right: auto;

            svg {
                width: 100%;
                height: 100%;
            }
        }

        p {
            margin-top: 3px;
            font-size: 12px;
        }

    }

}

@media screen and (max-width: 800px) {

    .footer_bar {
        display: flex;
        flex-direction: row;
        justify-content: space-around;
        align-items: center;
    }

}


</style>