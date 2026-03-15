import { ref, computed, onMounted, onUnmounted } from 'vue';

// 响应式配置 - 屏幕适配配置
const windowWidth = ref(typeof window !== 'undefined' ? window.innerWidth : 1920);
const windowHeight = ref(typeof window !== 'undefined' ? window.innerHeight : 1080);

// 断点配置
const breakpoints = {
    mobile: 768,
    tablet: 1000,
    desktop: 1300,
};

// 书籍网格配置
const bookGridConfig = computed(() => {
    const width = windowWidth.value;
    if (width <= breakpoints.mobile) {
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

// 是否为手机端
const isMobile = computed(() => windowWidth.value <= breakpoints.mobile);

// 是否为平板端
const isTablet = computed(() => windowWidth.value > breakpoints.mobile && windowWidth.value <= breakpoints.tablet);

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
    isMobile,
    isTablet,
    isDesktop,
    initResponsiveConfig,
};
