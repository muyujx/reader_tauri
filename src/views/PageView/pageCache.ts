import {PageItem} from "../../model/pageModel";
import {getBookPageList} from "../../apis/book";
import {preloadImage} from "../../service/imagePreLoad";
import {getLocalPage} from "../../apis/bookDownload";

enum PageCacheLoad {
    // 不操作缓存
    NONE,
    // 重置缓存
    RESET,
    // 添加在后面
    PUSH,
    // 添加在前面
    UNSHIFT
}


/**
 * 页面缓存处理类
 */
export class PageCache {

    /**
     * 默认的第一页页号
     */
    public static readonly FIRST_PAGE = 2;

    /**
     * 第一页显示书籍封面
     */
    public static readonly COVER_PAGE = 1;

    /**
     * 每次加载的页面数
     */
    private static readonly LOAD_SIZE = 5;


    /**
     * 重置缓存的时候加载的页面数
     * 这个值小于 LOAD_SIZE 以减少首次加载的耗时
     */
    private static readonly RESET_LOAD_SIZE = 2;

    /**
     * 最大页面缓存数量
     */
    private static readonly MAX_CACHE_SIZE = 30;

    /**
     * 触发页面加载的阈值,
     * 加载前一页时, 缓存前面剩余的页面数小于该值
     * 加载后一页时, 缓存后面剩余的页面数小于该值
     */
    private static readonly LOAD_THRESHOLD = 2;

    /**
     * 封面页
     */
    public static readonly COVER_PAGE_ITEM: Readonly<PageItem> = {
        content: '',
        title: '',
        page: 1,
        topChapter: 0
    };

    /**
     * 空页
     */
    public static readonly BLANK_PAGE_ITEM: Readonly<PageItem> = {
        content: '',
        title: '',
        page: 0,
        topChapter: 0
    };

    /**
     * 页面缓存列表
     */
    private cacheList = new Array<PageItem>();

    /**
     * 总页数
     */
    private totalPage = 0;

    /**
     * 书籍编号
     */
    private readonly bookId: number;

    /**
     * 是否使用本地下载的内容
     */
    private readonly useLocal: boolean;

    /**
     * 防止重复加载缓存
     */
    private pending = false;

    constructor(bookId: number, useLocal: boolean = false) {
        this.bookId = bookId;
        this.useLocal = useLocal;
    }

    public setTotalPage(totalPage: number): void {
        this.totalPage = totalPage;
    }

    /**
     * 获取对应的页面内容
     *
     * @param page 页号
     */
    public async getPage(page: number): Promise<PageItem | null> {

        // totalPage = 0 是没有获取到书籍信息时
        if (page < PageCache.FIRST_PAGE || (this.totalPage > 0 && page > this.totalPage)) {
            return Promise.resolve(null);
        }

        let pageSize = PageCache.LOAD_SIZE;

        const loadType = this.getLoadType(page, pageSize);

        let startRqPage = 0;

        switch (loadType) {
            case PageCacheLoad.NONE: {
                // 不操作缓存, 请求页面就在缓存中
                return this.resolvePage(page);
            }
            case PageCacheLoad.RESET: {
                // 重置缓存, 加载请求页面以及之后的页面
                startRqPage = page;
                pageSize = PageCache.RESET_LOAD_SIZE;
                break;
            }
            case PageCacheLoad.UNSHIFT: {
                // 向缓存前面加载页面
                const cacheFirst = this.cacheList[0].page;
                // 如果向前加载的页数以及不足 loadSize
                pageSize = Math.min(cacheFirst - PageCache.FIRST_PAGE, pageSize);
                startRqPage = cacheFirst - pageSize;
                break;
            }
            case PageCacheLoad.PUSH: {
                const cacheLast = this.cacheList[this.cacheList.length - 1].page;
                pageSize = Math.min(this.totalPage - cacheLast, pageSize);
                startRqPage = cacheLast + 1;
                // 向缓存后面加载页面
                break;
            }
        }

        if (this.pending) {
            return this.resolvePage(page);
        }
        this.pending = true;

        if (this.containsPage(page)) {
            this.loadPages(startRqPage, pageSize, loadType).finally(() => {
                this.pending = false;
            })
            return this.resolvePage(page);
        }

        try {
            await this.loadPages(startRqPage, pageSize, loadType);
            return await this.resolvePage(page);
        } finally {
            this.pending = false;
        }
    }


    private resolvePage(page: number): Promise<PageItem | null> {

        if (this.containsPage(page)) {
            const cacheFist = this.cacheList[0].page;
            return Promise.resolve(this.cacheList[page - cacheFist]);
        }

        return Promise.resolve(null);
    }

    private containsPage(page: number): boolean {

        if (this.cacheList.length == 0) {
            return false;
        }

        const cacheFist = this.cacheList[0].page;
        const cacheLast = this.cacheList[this.cacheList.length - 1].page;

        return !(page < cacheFist || page > cacheLast);
    }

    private async loadPages(startPage: number, pageSize: number, loadType: PageCacheLoad): Promise<void> {

        startPage = Math.max(startPage, PageCache.FIRST_PAGE);

        let pages: PageItem[];
        
        if (this.useLocal) {
            // 本地模式：优先从本地读取，本地没有的页面从网络获取
            const remotePages = await getBookPageList(this.bookId, startPage, pageSize);
            const remotePageMap = new Map<number, PageItem>();
            for (const page of remotePages) {
                remotePageMap.set(page.page, page);
            }
            
            pages = [];
            for (let i = 0; i < pageSize; i++) {
                const pageNum = startPage + i;
                if (pageNum > this.totalPage) break;
                // 优先从本地获取
                const localPage = await getLocalPage(this.bookId, pageNum);
                if (localPage) {
                    pages.push(localPage);
                } else if (remotePageMap.has(pageNum)) {
                    // 本地没有则使用网络数据
                    pages.push(remotePageMap.get(pageNum)!);
                }
            }
        } else {
            pages = await getBookPageList(this.bookId, startPage, pageSize);
        }
        
        if (loadType == PageCacheLoad.RESET) {
            this.cacheList = pages;
        } else if (loadType == PageCacheLoad.PUSH) {
            this.cacheList.push(...pages);
        } else {
            this.cacheList.unshift(...pages);
        }
        
        // 预加载图片
        preloadImage(pages);
        this.resizeCacheList(loadType);
    }

    private resizeCacheList(loadType: PageCacheLoad): void {
        if (this.cacheList.length > PageCache.MAX_CACHE_SIZE) {
            // 超过缓存的页面数量
            const count = this.cacheList.length - PageCache.MAX_CACHE_SIZE;

            if (loadType == PageCacheLoad.PUSH) {
                // 去除前面的
                this.cacheList = this.cacheList.slice(count, this.cacheList.length);
            } else {
                // 去除后面
                this.cacheList = this.cacheList.slice(0, this.cacheList.length - count);
            }
        }
    }


    /**
     * 获取加载类型
     * @param page 请求的页号
     * @param pageSize 获取页面数量
     */
    private getLoadType(page: number, pageSize: number): PageCacheLoad {

        if (this.cacheList.length == 0) {
            return PageCacheLoad.RESET
        }

        // 缓存中的页面范围 [cacheStartPage, cacheEndPage]
        const cacheStartPage = this.cacheList[0].page;
        const cacheEndPage = this.cacheList[this.cacheList.length - 1].page;

        // 不预先加载的页面范围 [thresholdStart, thresholdEnd]
        const thresholdStart = Math.min(cacheStartPage + PageCache.LOAD_THRESHOLD, cacheEndPage);
        const thresholdEnd = Math.max(cacheEndPage - PageCache.LOAD_THRESHOLD, cacheStartPage);

        // 加载后的页面范围 [afterLoadStart, afterLoadEnd]
        const afterLoadStart = Math.max(cacheStartPage - pageSize, PageCache.FIRST_PAGE);
        const afterLoadEnd = Math.min(cacheEndPage + pageSize, this.totalPage);

        if (page < afterLoadStart || page > afterLoadEnd) {
            // 加载后都无法覆盖请求的页面，直接重置缓存
            return PageCacheLoad.RESET;
        } else if (page < thresholdStart) {
            if (cacheStartPage > PageCache.FIRST_PAGE) {
                return PageCacheLoad.UNSHIFT;
            }
        } else if (page > thresholdEnd) {
            if (cacheEndPage < this.totalPage) {
                return PageCacheLoad.PUSH;
            }
        }

        return PageCacheLoad.NONE;
    }

}
