<template>

    <div class="tag-selector">

        <p>指定标签</p>

        <div class="tag"
             :class="tag.id == curTag ? 'active' : ''"
             v-for="tag in tags"
             :key="tag.id"
             @click="select(tag.id)"
        >

            {{ tag.name }}

        </div>

    </div>


</template>

<script setup lang="ts">

import {BookTag} from "../model/bookTag";

const props = defineProps<{
    tags: BookTag[],
    curTag?: number
}>();

const emit = defineEmits<{
    (e: 'change', newTag: number): void,
}>();

function select(tagId: number) {
    if (tagId != props.curTag) {
        emit("change", tagId);
    }
}


</script>


<style scoped lang="less">


.tag-selector {
    border-radius: 7px;
    background-color: var(--contrast);
    font-size: 14px;
}

.tag {
    height: 30px;
    line-height: 30px;
    padding: 0 5px;
    min-width: 70px;

    border-radius: 7px;

    &:hover {
        background-color: var(--selectBg);
        color: var(--selectFg);
    }

    &.active {
        color: var(--primary);
    }
}

</style>