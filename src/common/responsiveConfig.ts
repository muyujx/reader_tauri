import { ref, computed } from 'vue';

// 响应式配置 - 屏幕适配配置
const windowWidth = ref(typeof window !== 'undefined' ? window.innerWidth : 1920);
const windowHeight = ref(typeof window !== 'undefined' ? window.innerHeight : 1080);

// 断点配置
const breakpoints = {
    mobile: 414,
    pad: 768,
    tablet: 1000,
    desktop: 1300,
};

// 书籍网格配置
const bookGridConfig = computed(() => {
    const width = windowWidth.value;
    if (width <= breakpoints.mobile) {
        return {
            pageSize: 6,
            pagerCount: 3,
            columns: 3,
            showPaginationArrows: false,
        };
    } else if (width <= breakpoints.pad) {
        return {
            pageSize: 9,
            pagerCount: 5,
            columns: 3,
            showPaginationArrows: false,
        };
    } else if (width <= breakpoints.tablet) {
        return {
            pageSize: 12,
            pagerCount: 6,
            columns: 4,
            showPaginationArrows: true,
        };
    } else if (width <= breakpoints.desktop) {
        return {
            pageSize: 15,
            pagerCount: 7,
            columns: 5,
            showPaginationArrows: true,
        };
    }
    return {
        pageSize: 18,
        pagerCount: 7,
        columns: 6,
        showPaginationArrows: true,
    };
});

// 收藏页卡片尺寸配置
const favoriteCardConfig = {
    mobile: {
        cardHeight: 140,
        gap: 12,
        paginationHeight: 40,
        padding: 16,
        pagerCount: 3,
        minPageSize: 1,
        maxPageSize: 6,
    },
    pad: {
        cardHeight: 190,
        gap: 20,
        paginationHeight: 40,
        padding: 20,
        pagerCount: 5,
        minPageSize: 1,
        maxPageSize: 6,
    },
    desktop: {
        pageSize: 9,
        pagerCount: 7,
        showPaginationArrows: true,
    },
};

// 收藏页网格配置 - 返回配置信息，由组件根据实际容器高度计算 pageSize
const favoriteGridConfig = computed(() => {
    const width = windowWidth.value;
    
    if (width <= breakpoints.mobile) {
        const config = favoriteCardConfig.mobile;
        return {
            cardHeight: config.cardHeight,
            gap: config.gap,
            minPageSize: config.minPageSize,
            maxPageSize: config.maxPageSize,
            pagerCount: config.pagerCount,
            showPaginationArrows: false,
            // 预设计算的 pageSize 作为后备
            pageSize: config.minPageSize,
        };
    } else if (width <= breakpoints.pad) {
        const config = favoriteCardConfig.pad;
        return {
            cardHeight: config.cardHeight,
            gap: config.gap,
            minPageSize: config.minPageSize,
            maxPageSize: config.maxPageSize,
            pagerCount: config.pagerCount,
            showPaginationArrows: false,
            pageSize: config.minPageSize,
        };
    }
    return {
        pageSize: favoriteCardConfig.desktop.pageSize,
        pagerCount: favoriteCardConfig.desktop.pagerCount,
        showPaginationArrows: favoriteCardConfig.desktop.showPaginationArrows,
    };
});

// 是否为手机端
const isMobile = computed(() => windowWidth.value <= breakpoints.mobile);

// 是否为平板端
const isPad = computed(() => windowWidth.value > breakpoints.mobile && windowWidth.value <= breakpoints.pad);

// 是否为中等屏幕
const isTablet = computed(() => windowWidth.value > breakpoints.pad && windowWidth.value <= breakpoints.tablet);

// 是否为桌面端
const isDesktop = computed(() => windowWidth.value > breakpoints.tablet);

// 监听窗口大小变化
function initResponsiveConfig() {
    const handleResize = () => {
        windowWidth.value = window.innerWidth;
        windowHeight.value = window.innerHeight;
    };
    
    if (typeof window !== 'undefined') {
        window.addEventListener('resize', handleResize);
        return () => window.removeEventListener('resize', handleResize);
    }
    return () => {};
}

export {
    windowWidth,
    windowHeight,
    breakpoints,
    bookGridConfig,
    favoriteGridConfig,
    isMobile,
    isPad,
    isTablet,
    isDesktop,
    initResponsiveConfig,
};
