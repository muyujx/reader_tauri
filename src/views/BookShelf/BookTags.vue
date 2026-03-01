<template>

    <div class="book_tags"
         :class="{
            'ink-mod': inkMode.current
        }"
    >

        <div class="title">书籍标签:</div>

        <div class="tags">

            <el-check-tag class="tag"
                          type="info"
                          @change="changeTag(-1)"
                          :checked="curTag == -1">
                所有
            </el-check-tag>

            <el-check-tag class="tag"
                          type="info"
                          @change="changeTag(item.id)"
                          :checked="curTag == item.id"
                          v-for="item of tags">
                {{ item.name }}
            </el-check-tag>

        </div>

    </div>


</template>

<style scoped lang="less">

.book_tags {
    display: flex;
    align-items: center;
    margin-bottom: 2vh;
    margin-left: 40px;
    position: relative;
}

.title {
    height: 40px;
    margin-top: 10px;
    line-height: 40px;
    flex-shrink: 0;
    margin-right: 20px;
    font-weight: bold;
    color: var(--accent);
}

.tags {
    max-width: max(70vw, 600px);

    display: flex;
    flex-wrap: wrap;
    flex-shrink: 0;
    box-sizing: border-box;
    padding: 0 10px 0 10px;
    align-items: center;


    .tag {
        background-color: var(--second);
        height: 30px;
        line-height: 30px;
        padding: 0 10px 0 10px;
        border-radius: 5px;
        margin-left: 15px;
        margin-top: 10px;


        &.is-checked {
            background-color: var(--background);
            color: var(--primary);
        }
    }

}

.ink-mod {
    .title {
        color: var(--primary) !important;
    }
}

@media screen and (max-width: 800px) {

    .book_tags {
        margin-left: 30px;
    }

    .tags {
        width: 520px;
        font-size: 13px;
        --el-font-size-base: 12px;
    }

    .title {
        height: 40px;
        line-height: 40px;
        margin-right: 0;

        margin-top: 5px;
        align-self: start;
    }

}

</style>

<script setup lang="ts">

import {ref} from "vue";
import {BookTag} from "../../model/bookTag";
import {inkModeStore} from "../../store/inkMode";

const inkMode = inkModeStore();


const props = defineProps<{
    tag: number,
    tags: BookTag[]
}>();

const emit = defineEmits<{
    (e: "changeTag", tag: number): void
}>();

const curTag = ref(props.tag);

function changeTag(tag: number) {
    if (tag == curTag.value) {
        return;
    }
    curTag.value = tag;
    emit("changeTag", tag);
}


defineExpose({
    changeTag: (tag: number) => {
        curTag.value = tag;
    }
})

</script>
