import rq from "./request";
import {FavoriteBookList} from "../model/favoriteBook.ts";


/**
 * 获取书籍列表
 */
export function getFavoriteBookListAPi(page: number, pageSize: number): Promise<FavoriteBookList> {
    return rq.post({
        url: `/api/favorite/list`,
        body: {
            page,
            pageSize
        }
    });
}

/**
 * 添加收藏书籍
 * @param bookId 书籍 id
 */
export function addFavoriteApi(bookId: number): Promise<void> {
    return rq.post({
        url: `/api/favorite/add`,
        body: {
            bookId
        }
    });
}

/**
 * 删除收藏书籍
 * @param bookId 书籍 id
 */
export function delFavoriteApi(bookId: number) {
    return rq.post({
        url: `/api/favorite/delete`,
        body: {
            bookId
        }
    });
}

