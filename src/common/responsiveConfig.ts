import { computed } from 'vue';

// 固定配置 - 不再根据屏幕尺寸适配

// 书籍网格配置（固定值）
const bookGridConfig = computed(() => {
    return {
        pageSize: 18,
        pagerCount: 7,
        columns: 6,
        showPaginationArrows: true,
    };
});

// 收藏页网格配置（固定值）
const favoriteGridConfig = computed(() => {
    return {
        pageSize: 9,
        pagerCount: 7,
        showPaginationArrows: true,
    };
});

// 下载页网格配置（固定值，与收藏页保持一致）
const downloadGridConfig = computed(() => {
    return {
        pageSize: 9,
        pagerCount: 7,
        showPaginationArrows: true,
    };
});

// 是否为手机端（固定为 false）
const isMobile = computed(() => false);

// 是否为平板端（固定为 false）
const isPad = computed(() => false);

// 是否为中等屏幕（固定为 false）
const isTablet = computed(() => false);

// 是否为桌面端（固定为 true）
const isDesktop = computed(() => true);

// 初始化函数（空实现，保持兼容）
function initResponsiveConfig() {
    // 不再监听窗口大小变化
    return () => {};
}

export {
    bookGridConfig,
    favoriteGridConfig,
    downloadGridConfig,
    isMobile,
    isPad,
    isTablet,
    isDesktop,
    initResponsiveConfig,
};
