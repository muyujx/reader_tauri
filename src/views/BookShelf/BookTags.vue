<template>

    <div class="book_tags"
         :class="{
            'ink-mod': inkMode.current
        }"
    >
        <div class="tags_wrap">
            <span class="title">书籍标签:</span>

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

    </div>


</template>

<style scoped lang="less">

.book_tags {
    width: 100%;
    display: flex;
    justify-content: center;
    margin-bottom: 8px;
}

.tags_wrap {
    width: 100%;
    max-width: 1300px;
    padding: 0 10px;
    box-sizing: border-box;
}

.title {
    display: block;
    font-weight: bold;
    color: var(--accent);
    margin-bottom: 6px;
    font-size: 14px;
}

.tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    width: 100%;

    .tag {
        background-color: var(--second);
        height: 28px;
        line-height: 28px;
        padding: 0 12px;
        border-radius: 4px;
        font-size: 12px;

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
