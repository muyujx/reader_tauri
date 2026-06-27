<template>
    <div class="progress-ring-container" :style="{ width: size + 'px', height: size + 'px' }">
        <svg class="progress-ring" viewBox="0 0 34 34">
            <circle class="progress-ring__track"
                cx="17" cy="17" r="14"
                fill="none"
                :stroke="trackColor"
                :stroke-width="strokeWidth"
            />
            <circle class="progress-ring__circle"
                cx="17" cy="17" r="14"
                fill="none"
                :stroke="color"
                :stroke-width="strokeWidth"
                stroke-linecap="round"
                stroke-dasharray="87.96"
                :stroke-dashoffset="87.96 - (87.96 * percentage / 100)"
            />
        </svg>
        <span v-if="showText" class="progress-ring__text" :style="{ fontSize: fontSize + 'px', color: textColor }">
            {{ Math.round(percentage) }}%
        </span>
    </div>
</template>

<script setup lang="ts">
interface Props {
    /** 进度百分比 0-100 */
    percentage: number;
    /** 容器尺寸(px) */
    size?: number;
    /** 进度条宽度 */
    strokeWidth?: number;
    /** 进度条颜色 */
    color?: string;
    /** 轨道颜色 */
    trackColor?: string;
    /** 是否显示文字 */
    showText?: boolean;
    /** 文字大小 */
    fontSize?: number;
    /** 文字颜色 */
    textColor?: string;
}

withDefaults(defineProps<Props>(), {
    percentage: 0,
    size: 34,
    strokeWidth: 4,
    color: 'var(--el-color-primary)',
    trackColor: 'transparent',
    showText: true,
    fontSize: 10,
    textColor: 'var(--foreground)',
});
</script>

<style scoped lang="less">
.progress-ring-container {
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
}

.progress-ring {
    width: 100%;
    height: 100%;
    transform: rotate(-90deg);
}

.progress-ring__circle {
    transition: stroke-dashoffset 0.3s ease;
}

.progress-ring__text {
    position: absolute;
    font-weight: 700;
    white-space: nowrap;
}
</style>
