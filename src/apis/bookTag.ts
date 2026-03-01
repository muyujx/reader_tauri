import {BookTag} from "../model/bookTag";
import rq from "./request";

export function getAllTag(): Promise<BookTag[]> {
    return rq.get({
        url: '/api/book/tag/get/all'
    })
}


export function changeBookTagApi(bookId: number, tagId: number): Promise<void> {
    return rq.post({
        url: '/api/book/tag/change',
        body: {
            bookId,
            tagId
        }
    })
}