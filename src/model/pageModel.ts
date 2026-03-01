export interface PageItem {
    content: string,
    title: string,
    page: number,
    topChapter: number,
}

export interface BookInfo {
    bookName: string,
    totalPage: number,
    bigCoverPic: string,
    bookId: number,
    coverPic: string,
    tagId: number,
    favorite: boolean,
}

export interface BookShelfList {
    content: BookInfo[],
    total: number,
    totalPage: number
}