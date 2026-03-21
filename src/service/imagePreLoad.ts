import {PageItem} from "../model/pageModel";
import {addHost} from "../apis/request.ts";

/**
 * 预加载页面中的所有图片
 *
 * @param pages 书籍页面
 */
export function preloadImage(pages: PageItem[]) {
    for (let page of pages) {
        page.content = preloadImageByHtml(page.content);
    }
}

/**
 * 预加载 html 中的所有图片
 *
 * @param html html
 */
export function preloadImageByHtml(html: string): string {
    const div = document.createElement('body');
    div.innerHTML = html;

    const images = div.querySelectorAll("img");
    // @ts-ignore
    for (let image of images) {
        // 获取原始的 src 属性值（可能是相对路径或绝对路径）
        const originalSrc = image.getAttribute('src');
        if (originalSrc) {
            // 使用 addHost 处理图片 URL
            const processedSrc = addHost(originalSrc);
            // 设置处理后的 URL
            image.src = processedSrc;
            // 使用处理后的 URL 进行预加载
            preloadBySrc(processedSrc);
        }
    }

    return div.innerHTML;
}

/**
 * 预加载图片
 *
 * @param src 图片地址
 */
export function preloadBySrc(src: string) {
    if (src == null || src.length == 0) {
        return;
    }
    
    const img = new Image();
    
    // 添加错误处理
    img.onerror = () => {
        console.warn(`图片预加载失败: ${src}`);
        // 可以在这里添加重试逻辑或占位图处理
    };
    
    // 添加超时处理
    const timeout = 10000; // 10秒超时
    const timeoutId = setTimeout(() => {
        console.warn(`图片预加载超时: ${src}`);
        img.src = ''; // 停止加载
    }, timeout);
    
    img.onload = () => {
        clearTimeout(timeoutId);
        console.log(`图片预加载成功: ${src}`);
    };
    
    img.src = src;
}
