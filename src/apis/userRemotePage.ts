import rq from "./request";


/**
 * 获取远程书页进度
 */
export function getRemotePage(bookId: number): Promise<number> {
    return rq.get({
        url: `/api/user/page/get?bookId=${bookId}`,
    });
}