import rq from "./request";


/**
 * 更新阅读进度, 会记录阅读时间和阅读历史记录
 *
 * @param bookId 书籍 id
 * @param readingCost 阅读耗时 单位 s
 * @param page 当前阅读到的页数
 */
export function updateReadingProgressApi(bookId: number, page: number,
                                         readingCost: number): Promise<void> {
    return rq.post({
        url: `/api/book/progress/update`,
        body: {
            bookId,
            readingCost,
            page
        }
    });
}


export interface ReadingHistoryItem {
    dayStr: string
    bookList: BookHistory[]
}

export interface BookHistory {

    bookName: string

    coverPic: string

    readingCost: number

    startPage: number

    endPage: number

}


/**
 * 获取历史阅读记录
 *
 * @param startStr 起始时间
 * @param endStr 结束时间
 */
export function listReadingHistory(startStr: string,
                                   endStr: string): Promise<ReadingHistoryItem[]> {
    return rq.post({
        url: '/api/read/history/list',
        body: {
            startStr,
            endStr
        }
    })
}

