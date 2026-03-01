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
        // 直接修改 src 为完整 URL
        image.src = addHost(image.src);
        // 预加载图片
        preloadBySrc(image.currentSrc);
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
    new Image().src = src;
}
