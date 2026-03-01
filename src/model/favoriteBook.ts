export interface FavoriteBookInfo {
    bookName: string,
    totalPage: number,
    bigCoverPic: string,
    bookId: number,
    coverPic: string,
    tagId: number,
    page: number,
    lastRead: number,
    readingCost: number,
}

export interface FavoriteBookList {
    content: FavoriteBookInfo[],
    total: number,
    totalPage: number
}