import rq from "./request";
import {ChapterNote} from "../model/chapterNote";

export function getChapterNote(bookId: number, chapter: number): Promise<ChapterNote[]> {
    return rq.get({
        url: `/api/chapter/note?bookId=${bookId}&chapter=${chapter}`
    });
}
