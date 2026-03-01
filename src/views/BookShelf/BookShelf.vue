<template>

    <div class="shelf"
         @touchstart.stop="touchControl.touchstart"
         @touchmove.stop="touchControl.touchmove"
         @touchend.stop="touchControl.touchend"
         @contextmenu.prevent=""
    >

        <div class="mask"
             v-show="maskShow"
             @click="maskCancel"
        >
        </div>

        <div class="search">

            <el-autocomplete
                placement="bottom"
                v-model.trim="searchStr"
                :fetch-suggestions="searchOnType"
                placeholder="根据书名或作者名搜索"
                @blur="searchBookList"
                @keyup.enter="enterSearch"
                :trigger-on-focus="false"
                :teleported="false"
            >
                <template #default="{ item }">
                    <div v-html="item.label"></div>
                </template>

            </el-autocomplete>

        </div>

            <Tags
                ref="tagsComp"
                :tags="tags"
                :tag="tag" @change-tag="changeTag"/>

            <div class="books">


                <TagSelector class="tagSelector"
                             ref="tagSelector"
                             :tags="tags"
                             :cur-tag="curTag"
                             @change="changeBookTag"
                ></TagSelector>


                <el-popover
                    :disabled="titlePopoverDisable"
                    placement="right-start"
                    trigger="hover"
                    width="200"
                    :show-arrow="false"
                    :show-after="300"
                    :content="book.bookName"
                    v-for="book in booKList"
                    :key="book.bookId"
                    :popper-style="{
                    borderRadius: '10px'
                }"
                >

                    <template #reference>

                        <div class="book"
                             @click="toBookPage(book.bookId, book.favorite)"
                             @contextmenu.prevent="chooseTag(book, $event.target)"
                             :class="{
                                 'active': curBookId == book.bookId
                             }"
                        >

                            <div class="cover">

                                <div class="favorite-button"
                                     @click.stop="changeFavorite(book)"
                                >

                                    <el-icon v-if="!book.favorite">
                                        <Star/>
                                    </el-icon>

                                    <el-icon v-if="book.favorite" class="active">
                                        <StarFilled/>
                                    </el-icon>

                                </div>


                                <p class="tag"> {{ book.tagId == -1 ? '未分类' : tagMap.get(book.tagId)?.name }} </p>

                                <img :src="addHost(book.coverPic)" :alt="book.bookName"/>

                            </div>


                            <p class="name">{{ book.bookName }}</p>

                        </div>

                    </template>
                </el-popover>

            </div>

        <el-pagination
            v-model:current-page="page"
            v-model:page-size="pageSize"
            layout="prev, pager, next, jumper"
            :page-count="totalPage"
            @current-change="jumpToPage"
        />

    </div>

</template>

<style scoped src="./BookShelf.less" lang="less"/>

<script setup lang="ts">
import {useTemplateRef, ref} from "vue";
import {useRouter} from "vue-router";
import {BookInfo as ShelfBookInfo, BookShelfList} from "../../model/pageModel";
import {getBookInfoList} from "../../apis/book";
import {getLocalStorage, getLocalStorageInt, setLocalStorage} from "../../utils/localStorageUtil";
import hotkeys from "hotkeys-js";
import Tags from "./BookTags.vue";
import {TouchControl} from "../../service/touchControl";
import TagSelector from "../../components/TagSelector.vue";
import {BookTag} from "../../model/bookTag";
import {changeBookTagApi, getAllTag} from "../../apis/bookTag";
import {addHost} from "../../apis/request.ts";
import {userStore} from "../../store/userStore.ts";
import {UserRole} from "../../model/user.ts";
import {Star, StarFilled} from "@element-plus/icons-vue";
import {addFavoriteApi, delFavoriteApi} from "../../apis/favoriteBook.ts";
import {popErr, popSuccess} from "../../utils/message.ts";
import {loadingStore} from "../../store/loading.ts";
import {searchOnType} from "./BookShelf.ts";
import {
    downloadBook as downloadBookApi,
    resumeBookDownload,
    pauseBookDownload,
    getDownloadProgress,
    onDownloadProgress,
    BookInfo
} from "../../apis/bookDownload.ts";
import {CircleCheck} from "@element-plus/icons-vue";

const PAGE_LIST_INDEX = "page_list_index";
const PAGE_TAG_LOCAL = "page_tag_local";
const PAGE_LIST_SEARCH = "page_list_search";

const DEFAULT_FIRST_PAGE = 1;
const DEFAULT_TAG = -1;
const DEFAULT_SEARCH_STR = "";

const page = ref(1);
const pageSize = ref(18);
const totalPage = ref(1);
const booKList = ref(new Array<BookInfo>());
const router = useRouter();
const searchStr = ref('');
const tag = ref(-1);
const userInfo = userStore();

const tagSelector = useTemplateRef<InstanceType<typeof TagSelector>>('tagSelector');

const maskShow = ref(false);
const tags = ref<BookTag[]>([]);
const tagMap = new Map<number, BookTag>;
const titlePopoverDisable = ref(false);

const loading = loadingStore();
// 用来控制点击书籍的动画触发
const curBookId = ref(-1);

const curTag = ref(-1);
let tagBookId = -1;

console.log("---------- bookshelf setup ---------");


initPage();

function initPage(): void {
    // 从 localStorage 中获取上次访问的页
    page.value = getLocalStorageInt(PAGE_LIST_INDEX, DEFAULT_FIRST_PAGE);
    tag.value = getLocalStorageInt(PAGE_TAG_LOCAL, DEFAULT_TAG);
    searchStr.value = getLocalStorage(PAGE_LIST_SEARCH, DEFAULT_SEARCH_STR);
}

function toBookPage(bookId: number, favorite: boolean) {
    curBookId.value = bookId;

    setTimeout(() => {
        curBookId.value = -1;
    }, 100);

    setTimeout(() => {

        router.push({
            name: "Read",
            query: {
                "bookId": bookId,
                "favorite": String(favorite),
            }
        }).then();

        // 动画时间
    }, 200)
}


// 给书籍指定标签
function chooseTag(book: BookInfo, target: any) {

    if (userInfo.role != UserRole.Admin) {
        return;
    }

    let rec = target.getBoundingClientRect();
    // @ts-ignore
    const el = tagSelector.value.$el;
    const style = el.style;

    const width = 100;
    const margin = 20;

    titlePopoverDisable.value = true;
    maskShow.value = true;
    style.display = "block";

    if (rec.right + margin + width < window.innerWidth) {
        style.left = rec.right + margin + "px";
    } else {
        style.left = rec.left - el.offsetWidth - margin + "px";
    }

    if (rec.top + el.offsetHeight + margin < window.innerHeight) {
        style.top = rec.top + "px";
    } else {
        style.top = window.innerHeight - el.offsetHeight - margin + "px";
    }

    curTag.value = book.tagId;
    tagBookId = book.bookId;
}


function changeBookTag(tagId: number) {

    changeBookTagApi(tagBookId, tagId).then(() => {
        // 刷新一下书籍列表
        getBookList();
    })

    maskCancel();
}

function maskCancel() {
    // 取消标签选择的弹窗
    // @ts-ignore
    tagSelector.value.$el.style.display = "none";
    maskShow.value = false;
    titlePopoverDisable.value = false;
}


function jumpToPage(pageIdx: number) {
    if (pageIdx < 1 || (totalPage.value != 0 && pageIdx > totalPage.value)) {
        return;
    }
    page.value = pageIdx;
    getBookList();

    setLocalStorage(PAGE_LIST_INDEX, page.value.toString());
    setLocalStorage(PAGE_TAG_LOCAL, tag.value.toString());
    setLocalStorage(PAGE_LIST_SEARCH, searchStr.value.toString());
}


// 上一次搜索的字符串
let lastSearch: string | null = null;
const tagsComp = useTemplateRef<InstanceType<typeof Tags> | null>("tagsComp");

function searchBookList() {

    if (lastSearch == searchStr.value) {
        return;
    }

    // 搜索字符串由空变变有内容, 修改 tag 为所有
    if ((lastSearch == null || lastSearch.length == 0) && searchStr.value.length > 0) {
        tag.value = -1;
        tagsComp.value?.changeTag(-1);
    }

    lastSearch = searchStr.value;
    jumpToPage(1);
}

function enterSearch(event: KeyboardEvent) {
    // @ts-ignore
    event?.target?.blur();
}

function next() {
    jumpToPage(page.value + 1);
}

function pre() {
    jumpToPage(page.value - 1)
}

const touchControl = new TouchControl();
touchControl.onSwipeLeft(next);
touchControl.onSwipeRight(pre);

function getBookList() {

    console.log("----------- getBookList --------");


    loading.show();

    getBookInfoList(page.value, pageSize.value, searchStr.value, tag.value)
        .then((bookInfoList: BookShelfList) => {
            totalPage.value = bookInfoList.totalPage;
            booKList.value = bookInfoList.content;
        })
        .finally(() => {
            loading.hide();
        });
}

function changeTag(curTag: number) {
    if (curTag == tag.value) {
        return;
    }
    tag.value = curTag;
    jumpToPage(1);
}


function changeFavorite(book: BookInfo) {
    let bookId = book.bookId;

    let res: Promise<void>;

    if (book.favorite) {
        res = delFavoriteApi(bookId)
            .then(() => {
                popSuccess("取消收藏成功")
            })
            .catch(() => {
                popErr("取消收藏失败")
            })
    } else {
        res = addFavoriteApi(bookId)
            .then(() => {
                popSuccess("收藏成功")
            })
            .catch(() => {
                popErr("收藏失败")
            });
    }

    res.finally(() => {
        getBookList();
    })
}

function enter() {

    hotkeys('left, up, a', 'book-shelf', pre);
    hotkeys('right, down, d, f', 'book-shelf', next);
    hotkeys.setScope('book-shelf');

    getBookList();
    // 获取书籍标签
    getAllTag().then(res => {
        for (let tag of res) {
            tagMap.set(tag.id, tag);
        }
        tags.value = res;
    });

}

function leave() {
    hotkeys.deleteScope('book-shelf');
}

defineExpose({
    'enter': enter,
    'leave': leave
})

</script>



