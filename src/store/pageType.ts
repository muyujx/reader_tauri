import {defineStore} from "pinia";


export enum PageType {

    /**
     * 书籍列表页面
     */
    BOOK_SHELF,

    /**
     * 书页
     */
    PAGE_VIEW

}


export const pageTypeStore = defineStore("pageType", (): { current: PageType } => {
    return {
        current: PageType.BOOK_SHELF
    }
});

