<template>

    <div class="favorite">

        <div class="empty-notify"
             :class="{
                'active': empty
            }"
        >
            <el-icon>
                <MostlyCloudy/>
            </el-icon>
            <p>还没有收藏书籍</p>
        </div>


        <div class="books">


            <div class="book"
                 :content="book.bookName"
                 v-for="book in bookList"
                 :key="book.bookId"
            >

                <div class="cover"
                     @click="toBookPage(book)"
                >
                    <img :src="addHost(book.coverPic)" :alt="book.bookName"/>

                </div>


                <div class="detail">

                    <div class="detail-content">
                        <p class="name">{{ book.bookName }}</p>

                        <div class="item">
                            <p>阅读进度:</p>

                            <div class="detail_item_content">
                                <p>{{ book.lastRead == 0 ? 0 : book.page }} / {{ book.totalPage }} 页</p>

                                <el-progress
                                    :text-inside="true"
                                    :stroke-width="15"
                                    :format="num => `${num == 0 ? '0' : num.toFixed(2)}%`"
                                    :percentage="book.lastRead == 0 ? 0 : (book.page / book.totalPage * 100)"
                                />

                            </div>

                        </div>

                        <div class="item">
                            <p>阅读时间:</p>
                            <p>{{ readCost(book.readingCost) }}</p>
                        </div>

                        <div class="item">
                            <p>上次阅读:</p>
                            <p>{{
                                    book.lastRead == 0 ? '未阅读' : getLastRead(book.lastRead)
                                }}</p>
                        </div>
                    </div>

                    <div class="action-buttons">
                        <div class="download-book"
                             @click.stop="downloadBook(book)"
                             v-if="!downloadStatus[book.bookId]?.downloading && !downloadStatus[book.bookId]?.downloaded && !downloadStatus[book.bookId]?.hasPartialDownload"
                        >
                            <el-icon>
                                <Download/>
                            </el-icon>
                        </div>

                        <div class="download-progress clickable"
                             @click.stop="handleDownloadClick(book)"
                             v-else-if="downloadStatus[book.bookId]?.hasPartialDownload && !downloadStatus[book.bookId]?.downloading && !downloadStatus[book.bookId]?.paused"
                             title="点击继续下载"
                        >
                            <el-progress
                                type="circle"
                                :width="24"
                                :percentage="downloadStatus[book.bookId]?.progress || 0"
                                :show-text="false"
                            />
                            <span class="progress-text">{{ downloadStatus[book.bookId]?.progress || 0 }}%</span>
                        </div>

                        <div class="download-progress clickable"
                             @click.stop="handleDownloadClick(book)"
                             v-else-if="downloadStatus[book.bookId]?.paused"
                             title="点击继续下载"
                        >
                            <el-progress
                                type="circle"
                                :width="24"
                                :percentage="downloadStatus[book.bookId]?.progress || 0"
                                :show-text="false"
                            />
                            <span class="progress-text">{{ downloadStatus[book.bookId]?.progress || 0 }}%</span>
                        </div>

                        <div class="download-progress clickable"
                             @click.stop="handleDownloadClick(book)"
                             v-else-if="downloadStatus[book.bookId]?.downloading"
                             title="点击暂停"
                        >
                            <el-progress
                                type="circle"
                                :width="24"
                                :percentage="downloadStatus[book.bookId]?.progress || 0"
                                :show-text="false"
                            />
                            <span class="progress-text">{{ downloadStatus[book.bookId]?.progress || 0 }}%</span>
                        </div>

                        <div class="download-complete"
                             v-else-if="downloadStatus[book.bookId]?.downloaded"
                        >
                            <el-icon color="#67c23a">
                                <CircleCheck/>
                            </el-icon>
                        </div>

                        <div class="cancel-favorite"
                             @click="cancelFavorite(book.bookId)"
                        >

                            <el-icon>
                                <StarFilled/>
                            </el-icon>

                        </div>
                    </div>

                </div>

            </div>

        </div>


        <el-pagination
            v-model:current-page="page"
            v-model:page-size="pageSize"
            :layout="paginationLayout"
            :page-count="totalPage"
            :pager-count="pagerCount"
            @current-change="jumpToPage"
        />

    </div>


</template>

<style scoped lang="less" src="./FavoriteBook.less"/>

<script setup lang="ts">
import {addHost} from "../../apis/request.ts";
import {onMounted, onUnmounted, ref, computed} from "vue";
import {useRouter} from "vue-router";
import {BookTag} from "../../model/bookTag.ts";
import {getAllTag} from "../../apis/bookTag.ts";
import {delFavoriteApi, getFavoriteBookListAPi} from "../../apis/favoriteBook.ts";
import {FavoriteBookInfo, FavoriteBookList} from "../../model/favoriteBook.ts";
import {MostlyCloudy, StarFilled, Download, CircleCheck} from "@element-plus/icons-vue";
import {popErr, popSuccess} from "../../utils/message.ts";
import {loadingStore} from "../../store/loading.ts";
import hotkeys from "hotkeys-js";
import windowSizeListener from "../../service/windowSize.ts";
import {formatDistanceToNow} from 'date-fns';
import {zhCN} from 'date-fns/locale';
import {favoriteGridConfig, initResponsiveConfig} from "../../common/responsiveConfig.ts";
import {
    downloadBook as downloadBookApi,
    resumeBookDownload,
    pauseBookDownload,
    cancelBookDownload,
    getDownloadProgress,
    onDownloadProgress,
    BookInfo
} from "../../apis/bookDownload.ts";

const bookList = ref(new Array<FavoriteBookInfo>());
const page = ref(1);
const pageSize = ref(12);
const pagerCount = computed(() => favoriteGridConfig.value.pagerCount);
const totalPage = ref(1);

// 获取每页数量
function getPageSize(): number {
    return favoriteGridConfig.value.pageSize;
}

// 手机端隐藏翻页按钮，用滑动替代
const paginationLayout = computed(() => {
    return favoriteGridConfig.value.showPaginationArrows ? 'prev, pager, next, jumper' : 'pager';
});

const router = useRouter();
const tagMap = new Map<number, BookTag>;
const tags = ref<BookTag[]>([]);
const empty = ref(false);
const loading = loadingStore();

interface DownloadStatus {
    downloading: boolean;
    downloaded: boolean;
    progress: number;
    hasPartialDownload?: boolean;
    paused?: boolean;
}

const downloadStatus = ref<Record<number, DownloadStatus>>({});

// 监听窗口大小变化，修改 pageSize
const onWindowSizeChange = () => {
    const curPageSize = getPageSize();
    if (pageSize.value !== curPageSize) {
        pageSize.value = curPageSize;
        page.value = 1;
        getBookList();
    }
};
windowSizeListener.on(onWindowSizeChange);
onMounted(() => {
    initResponsiveConfig();
});
onUnmounted(() => {
    console.log("----- FavoriteBook Unmounted ---");
    windowSizeListener.delete(onWindowSizeChange);
})

function getBookList() {

    loading.show();

    getFavoriteBookListAPi(page.value, pageSize.value)
        .then(async (bookInfoList: FavoriteBookList) => {

            empty.value = (bookInfoList.totalPage == 0);

            totalPage.value = bookInfoList.totalPage;
            bookList.value = bookInfoList.content;

            for (const book of bookInfoList.content) {
                const progressInfo = await getDownloadProgress(book.bookId);
                
                // 根据 exists 字段判断书籍是否存在
                const isBookExists = progressInfo.exists;
                const isFullyDownloaded = isBookExists && progressInfo.downloadedPages >= progressInfo.totalPage;
                const hasPartialDownload = isBookExists && progressInfo.downloadedPages > 0 && !isFullyDownloaded;
                const progress = isBookExists && progressInfo.totalPage > 0
                    ? Math.floor((progressInfo.downloadedPages / progressInfo.totalPage) * 100)
                    : 0;
                
                if (!downloadStatus.value[book.bookId]) {
                    downloadStatus.value[book.bookId] = {
                        downloading: false,
                        downloaded: isFullyDownloaded,
                        progress: progress,
                        hasPartialDownload: hasPartialDownload
                    };
                } else if (!downloadStatus.value[book.bookId].downloaded) {
                    downloadStatus.value[book.bookId].downloaded = isFullyDownloaded;
                    downloadStatus.value[book.bookId].progress = progress;
                    downloadStatus.value[book.bookId].hasPartialDownload = hasPartialDownload;
                }
            }
        })
        .finally(() => {
            loading.hide();
        });
}

async function handleDownloadClick(book: FavoriteBookInfo) {
    const status = downloadStatus.value[book.bookId];
    
    // 下载中 - 暂停
    if (status?.downloading && !status.paused) {
        await pauseBookDownload(book.bookId);
        downloadStatus.value[book.bookId].paused = true;
        popSuccess('已暂停');
        return;
    }
    
    // 暂停中 - 继续
    if (status?.paused) {
        downloadStatus.value[book.bookId].paused = false;
        await resumeBookDownload(book.bookId);
        return;
    }
    
    // 其他情况 - 开始/继续下载
    await downloadBook(book);
}

async function downloadBook(book: FavoriteBookInfo) {
    console.log(book);
    if (downloadStatus.value[book.bookId]?.downloading && !downloadStatus.value[book.bookId]?.paused) {
        return;
    }

    const hasPartial = downloadStatus.value[book.bookId]?.hasPartialDownload;
    const isResume = hasPartial && downloadStatus.value[book.bookId]?.progress > 0;
    
    // 设置为下载中，继续下载时保持当前进度
    downloadStatus.value[book.bookId] = {
        downloading: true,
        downloaded: false,
        progress: isResume ? downloadStatus.value[book.bookId].progress : 0,
        hasPartialDownload: hasPartial || false,
        paused: false
    };

    const bookInfo: BookInfo = {
        bookId: book.bookId,
        bookName: book.bookName,
        totalPage: book.totalPage,
        coverPic: book.coverPic,
        bigCoverPic: book.bigCoverPic,
        tagId: book.tagId
    };

    try {
        if (hasPartial) {
            await resumeBookDownload(book.bookId);
        } else {
            await downloadBookApi(bookInfo);
        }
        popSuccess('下载完成');
        const progressInfo = await getDownloadProgress(book.bookId);
        const isFullyDownloaded = progressInfo.exists && progressInfo.downloadedPages >= progressInfo.totalPage;
        const progress = progressInfo.exists && progressInfo.totalPage > 0 
            ? Math.floor((progressInfo.downloadedPages / progressInfo.totalPage) * 100) 
            : 100;
        downloadStatus.value[book.bookId] = {
            downloading: false,
            downloaded: isFullyDownloaded,
            progress: progress,
            hasPartialDownload: false,
            paused: false
        };
    } catch (error) {
        const prevProgress = downloadStatus.value[book.bookId]?.progress || 0;
        downloadStatus.value[book.bookId] = {
            downloading: false,
            downloaded: false,
            progress: prevProgress,
            hasPartialDownload: hasPartial || false,
            paused: false
        };
        popErr('下载失败');
    }
}

onDownloadProgress((progress) => {
    if (downloadStatus.value[progress.bookId]) {
        downloadStatus.value[progress.bookId].progress = progress.progress;
    }
});


function toBookPage(book: FavoriteBookInfo) {

    setTimeout(() => {

        router.push({
            name: "Read",
            query: {
                "bookId": book.bookId,
                "remotePage": book.page,
                "favorite": "true"
            }
        }).then();

        // 动画时间
    }, 200)
}

function jumpToPage(pageIdx: number) {

    console.log(`pageIdx = ${pageIdx}, totalPage = ${totalPage.value}, page = ${page.value}`);

    if (pageIdx < 1 || (totalPage.value != 0 && pageIdx > totalPage.value)) {
        return;
    }
    page.value = pageIdx;
    getBookList();
}

function cancelFavorite(bookId: number) {
    delFavoriteApi(bookId)
        .then(() => {
            popSuccess("取消收藏");
        })
        .catch((() => {
            popErr("取消收藏失败")
        }))
        .finally(() => {
            getBookList();
        })
}

function getLastRead(lastReadTime: number): string {
    return formatDistanceToNow(new Date(lastReadTime),
        {
            addSuffix: true,
            locale: zhCN,
        });
}

function readCost(minutes: number): string {
    minutes = Math.floor(minutes / 60);
    if (minutes < 60) {
        return `${minutes} 分钟`
    }
    let hour = Math.floor(minutes / 60);
    let minute = Math.floor(minutes % 60);
    if (minute == 0) {
        return `${hour} 小时`
    }
    return `${hour} 小时 ${minute} 分钟`;
}

function enter() {
    console.log("--- FavoriteBook Page Enter ----");

    hotkeys('left, a, s, page up', 'favorite', () => jumpToPage(page.value - 1));
    hotkeys('right, f, d, page down', 'favorite', () => jumpToPage(page.value + 1));
    hotkeys.setScope('favorite');

    // 初始化每页数量
    pageSize.value = getPageSize();

    // 获取书籍标签
    getAllTag().then(res => {
        for (let tag of res) {
            tagMap.set(tag.id, tag);
        }
        tags.value = res;
    });

    getBookList();
}

function leave() {
    console.log("--- FavoriteBook Page Leave ----");
    hotkeys.deleteScope('favorite');
}

defineExpose({
    'enter': enter,
    'leave': leave,
})

</script>


