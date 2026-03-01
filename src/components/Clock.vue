<template>

    <div class="clock">
        {{ timeStr }}
    </div>

</template>


<script setup lang="ts">
import { format } from 'date-fns';
import {ref} from "vue";

const timeStr = ref(getTime());

function getTime(): string {
    return format(new Date(), 'HH:mm');
}

setInterval(() => {
    timeStr.value = getTime();
}, 5000);


</script>

<style scoped lang="less">

.clock {
    color: var(--text);
    width: 200px;
    font-size: 16px;
    user-select: none;
    position: absolute;
    bottom: 0;
}

</style>