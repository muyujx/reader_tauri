import {ipcInvoke, ipcOn} from "../utils/ipcUtil.ts";
import ipcChannel from "../common/ipcChannel.ts";

export interface BookInfo {
    bookId: number;
    bookName: string;
    totalPage: number;
    coverPic: string;
    bigCoverPic: string;
    tagId: number;
}

export interface PageItem {
    content: string;
    title: string;
    page: number;
    topChapter: number;
}

export interface DownloadProgress {
    exists: boolean;
    downloadedPages: number;
    totalPage: number;
}

export function downloadBook(bookInfo: BookInfo): Promise<{ success: boolean, bookId: number }> {
    return ipcInvoke(ipcChannel.bookDownload, {
        bookId: bookInfo.bookId,
        bookName: bookInfo.bookName,
        totalPage: bookInfo.totalPage,
        coverPic: bookInfo.coverPic,
        bigCoverPic: bookInfo.bigCoverPic,
        tagId: bookInfo.tagId,
    });
}

/**
 * 继续下载书籍（断点续传）
 */
export function resumeBookDownload(bookId: number): Promise<{ success: boolean, bookId: number, resumed?: boolean }> {
    return ipcInvoke(ipcChannel.bookResumeDownload, { bookId });
}

/**
 * 暂停下载
 */
export function pauseBookDownload(bookId: number): Promise<{ success: boolean }> {
    return ipcInvoke(ipcChannel.bookPauseDownload, { bookId });
}

/**
 * 取消下载
 */
export function cancelBookDownload(bookId: number): Promise<{ success: boolean }> {
    return ipcInvoke(ipcChannel.bookCancelDownload, { bookId });
}

/**
 * 获取书籍下载进度
 */
export function getDownloadProgress(bookId: number): Promise<DownloadProgress> {
    return ipcInvoke(ipcChannel.bookGetDownloadProgress, { bookId });
}

export function getLocalPage(bookId: number, page: number): Promise<PageItem | null> {
    return ipcInvoke(ipcChannel.bookGetPage, { bookId, page });
}

export function getLocalBookInfo(bookId: number): Promise<BookInfo | null> {
    return ipcInvoke(ipcChannel.bookGetInfo, { bookId });
}

export function deleteLocalBook(bookId: number): Promise<{ success: boolean }> {
    return ipcInvoke(ipcChannel.bookDelete, { bookId });
}

export function getLocalImage(bookId: number, imageUrl: string): Promise<{ data: string, mimeType: string } | null> {
    return ipcInvoke(ipcChannel.bookGetLocalImage, { bookId, imageUrl });
}

export function onDownloadProgress(callback: (progress: DownloadProgress) => void): void {
    ipcOn(ipcChannel.bookDownloadProgress, (...args: any[]) => {
        callback(args[0]);
    });
}

/**
 * 获取所有已下载的书籍列表
 */
export interface DownloadedBookInfo {
    bookId: number;
    bookName: string;
    totalPage: number;
    coverPic: string;
    bigCoverPic: string;
    tagId: number;
    createTime: number;
    downloadedPages: number;
    progress: number;
    // 阅读进度相关字段
    readPage: number;      // 当前阅读到的页码
    lastRead: number;      // 上次阅读时间戳
    readingCost: number;   // 累计阅读耗时（秒）
}

export function getDownloadedBookList(): Promise<DownloadedBookInfo[]> {
    return ipcInvoke(ipcChannel.bookGetAllList, null);
}

/**
 * 分页获取已下载的书籍列表
 */
export interface DownloadedBookListResult {
    content: DownloadedBookInfo[];
    total: number;
    totalPage: number;
}

export function getDownloadedBookListByPage(page: number, pageSize: number): Promise<DownloadedBookListResult> {
    return ipcInvoke(ipcChannel.bookGetListByPage, { page, pageSize });
}

 /**
 * 更新本地书籍阅读进度
 * 
 * @param bookId 书籍 ID
 * @param readPage 当前阅读到的页码
 * @param readingCost 累计阅读耗时（秒）
 */
export function updateLocalReadProgress(bookId: number, readPage: number, readingCost: number): Promise<{ success: boolean }> {
    return ipcInvoke(ipcChannel.bookUpdateReadProgress, { bookId, readPage, readingCost });
}

export function saveLocalPage(bookId: number, pageIdx: number, content: string, title: string, topChapter: number): Promise<{ success: boolean }> {
    return ipcInvoke(ipcChannel.bookSavePage, { 
        bookId, 
        pageIdx, 
        content, 
        title, 
        topChapter 
    });
}
