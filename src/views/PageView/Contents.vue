<template>

    <div class="contents"
         :class="{
            show: showContents
         }">
        <div class="contents_header">

            <h3 @click="emit('skipCoverPage')">{{ bookName }}</h3>

            <el-icon
                class="close_contents"
                @click="toggleContents"
            >
                <Fold/>
            </el-icon>
        </div>


        <div class="contents_body" ref="contentsBody">

            <div
                class="contents_item"
                :class="'level' + item.level, (curChapter?.startPage == item.startPage) ? 'active' : ''"
                v-for="item in contents"
                @click="skipChapter(item)">
                <p :style="`padding-left: ${item.level * 20}px`">{{ item.label }}</p>
                <p>{{ item.startPage }}</p>
            </div>

        </div>

    </div>


    <el-icon
        class="open_contents"
        :class="{
            show: !showContents
         }"
        @click="toggleContents">
        <Expand/>
    </el-icon>

    <div class="mask"
         @click="toggleContents"
         :class="{
            'active': showContents
         }"
    />

</template>

<script setup lang="ts">
import {nextTick, onBeforeUnmount, onMounted, ref, Ref, useTemplateRef} from "vue";
import {ContentsItem} from "../../model/contentsModel";
import {getContents} from "../../apis/book";
import {getLocalStorageBoolean, setLocalStorage} from "../../utils/localStorageUtil";
import {Expand, Fold} from '@element-plus/icons-vue';
import hotkeys from "hotkeys-js";
import {binarySearch} from "../../utils/util";

const props = defineProps<{
    bookId: number,
    bookName: string
}>();

const emit = defineEmits<{
    (e: 'skipChapter', page: number): void,
    (e: 'skipCoverPage'): void,
    (e: 'open'): void,
    (e: 'close'): void
}>();

const contentsStateKey = `content_state_${props.bookId}`;

const contents = ref(new Array<ContentsItem>());

// 是否显示目录
const showContents = ref(getLocalStorageBoolean(contentsStateKey, true));
notifyShow();

// 获取目录数据
getContents(props.bookId).then((contentList: ContentsItem[]) => {
    contents.value = contentList;
});

function notifyShow() {
    if (showContents.value) {
        emit('open');
    } else {
        emit('close');
    }
}

/**
 * 开关目录
 */
function toggleContents(evt: any) {
    evt.preventDefault();
    showContents.value = !showContents.value;

    if (showContents.value) {
        scrollToMid();
    }

    setLocalStorage(contentsStateKey, showContents.value.toString());
    notifyShow();
}

hotkeys('tab', toggleContents);
onBeforeUnmount(() => {
    hotkeys.unbind('tab');
});


/**
 * 跳转章节第一页
 */
function skipChapter(item: ContentsItem) {
    emit('skipChapter', item.startPage);
}

// 当前页数所在的章节, 用来标示当前页所在章节
const curChapter: Ref<ContentsItem | null> = ref(null);

defineExpose({

    show: () => {
        showContents.value = true;
    },

    close: () => {
        showContents.value = false;
    },


    // 修改当前页数
    changePage: (page: number) => {
        let idx = binarySearch(contents.value, page, (a, b) => {
            return a.startPage - b;
        });
        if (idx == -1) {
            curChapter.value = null;
        } else {
            curChapter.value = contents.value[idx];

            nextTick(() => {
                scrollToMid();
            });
        }
    }
})

const contentsBody = useTemplateRef("contentsBody");

onMounted(() => {
    scrollToMid();
});

function scrollToMid() {
    // @ts-ignore
    const container = contentsBody.value?.$el || contentsBody.value;
    if (!container) {
        return;
    }
    // 获取当前 active 元素
    const activeEl = container.querySelector('.active');
    if (!activeEl) {
        return;
    }

    const offsetTop = activeEl.offsetTop;
    container.scrollTo({
        top: offsetTop - container.clientHeight / 2,
        behavior: 'smooth'
    });
}


</script>


<style scoped lang="less">


.contents {
    --contents-width: 500px;

    height: 100%;
    flex-shrink: 0;
    width: var(--contents-width);


    user-select: none;
    display: flex;
    box-sizing: border-box;
    color: var(--foreground);
    background-color: var(--background);
    flex-direction: column;
    transition: 500ms;
    backface-visibility: hidden;
    border-radius: 0 20px 20px 0;

    z-index: 100;

    position: absolute;
    transform: translateX(calc(0px - var(--contents-width)));

    &.show {
        transform: translateX(0);
    }
}

.contents_item:hover, .contents_item.active {
    color: var(--accent);
}

.open_contents {
    width: 50px;
    height: 50px;
    box-sizing: border-box;
    color: var(--text);
    position: fixed;
    top: 3%;
    left: 0;
    transition: left 100ms, opacity 100ms ease;
    transition-delay: 0ms;
    opacity: 0;
    z-index: 20;

    svg {
        margin-right: 10px;
        width: 40px;
        height: 40px;
    }

    &.show {
        transition-delay: 600ms;
        opacity: 1;
    }

}

.contents_header {
    width: 100%;
    min-height: 70px;
    display: flex;
    justify-content: center;
    align-items: center;

    h3 {
        color: var(--accent);
        width: 400px;
        font-size: 20px;
        line-height: 40px;
        box-sizing: border-box;
        text-align: center;
        margin-left: 20px;
        padding: 10px 0;
    }

    .close_contents {
        width: 70px;
        height: 70px;
        color: var(--text);

        svg {
            margin: auto;
            height: 35px;
            width: 35px;
        }
    }

}

.contents_body::-webkit-scrollbar {
    width: 10px;
    border-radius: 10px;
}

.contents_body::-webkit-scrollbar-thumb {
    border-radius: 6px;
    background-color: var(--second);
    border: 1px solid var(--border);
    box-sizing: border-box;
}

.contents_body::-webkit-scrollbar-thumb:hover {
    background-color: var(--active);
}

.contents_body {
    width: 100%;
    height: 100%;
    flex: 0 0 1;
    box-sizing: border-box;
    overflow: auto;
    padding: 10px 10px;
}

.contents_item {
    font-size: 14px;
    box-sizing: border-box;

    display: flex;
    height: 50px;
    align-content: center;
    justify-content: space-between;

    padding: 10px;
}

.contents_item:last-child {
    margin-bottom: 10px;
}

.level0 {
    font-weight: bold;
}


.contents_item > p {
    line-height: 30px;
    margin: 0;
}

.contents_item > p:first-child {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    box-sizing: border-box;
    padding-right: 10px;
}

.contents_item > p:last-child {
    width: 35px;
}

.ink-mod.contents {
    &, * {
        transition: all 0s;
        transition-delay: 0s;
    }

    .contents_header {
        h3 {
            color: var(--primary);
        }
    }

    .contents_item:hover {
        color: var(--primary);
    }

}

.mask {
    position: fixed;
    left: 0;
    top: 0;
    display: none;
    width: 100%;
    height: 100%;
    z-index: 11;
    background-color: unset;
}

@media screen and (max-width: 800px) {
    .contents {
        --contents-width: 350px;
        position: fixed;
        left: 0;
        top: 0;
    }

    .close_contents {
        display: none;
    }

    .contents_header {
        h3 {
            font-size: 14px;
            width: 300px;
        }
    }

    .contents_body {
        padding: 5px;
    }

    .contents_item {

        & > p:last-child {
            width: 20px;
        }
    }

    .open_contents {
        display: none;
    }

    .mask.active {
        display: block;
    }
}

</style>