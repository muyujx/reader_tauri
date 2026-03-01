import {ChapterNote} from "../model/chapterNote";
import {getChapterNote} from "../apis/chapterNote";

export class ChapterNoteService {

    /**
     * 当前缓存章节号
     * @private
     */
    private curChapter = -1;

    /**
     * 当前的注释缓存
     * @private
     */
    private noteMap: Map<number, ChapterNote> | null = null;

    private bookId: number = -1;

    constructor(bookId: number) {
        this.bookId = bookId;
    }

    public getChapterNote(chapter: number, noteId: number): Promise<string> {
        if (chapter != this.curChapter) {
            return getChapterNote(this.bookId, chapter)
                .then(resList => {
                    this.noteMap = new Map<number, ChapterNote>();
                    for (const item of resList) {
                        this.noteMap.set(item.noteId, item);
                    }
                })
                .then(() => {
                    return this.getNote(noteId);
                })
        }

        return Promise.resolve(this.getNote(noteId));
    }

    private getNote(noteId: number): string {
        const chapterNote = this.noteMap?.get(noteId);
        if (chapterNote == undefined) {
            return "";
        }
        return chapterNote.note;
    }


}