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


        <div class="books" ref="booksContainer">


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
                        <!-- 统一的下载状态视图：idle / loading / paused / done 四态 -->
                        <!-- idle: 未下载，显示下载图标 -->
                        <div class="download-book"
                             @click.stop="handleDownloadClick(book)"
                             v-if="viewMode(book.bookId) === 'idle'"
                             title="点击下载"
                        >
                            <el-icon>
                                <Download/>
                            </el-icon>
                        </div>

                        <!-- loading: 下载中，圆形进度，点击暂停 -->
                        <div class="download-progress clickable"
                             @click.stop="handleDownloadClick(book)"
                             v-else-if="viewMode(book.bookId) === 'loading'"
                             title="点击暂停"
                        >
                            <el-progress
                                type="circle"
                                :width="30"
                                :stroke-width="3"
                                :percentage="downloadStatus[book.bookId]?.progress || 0"
                            />
                        </div>

                        <!-- paused: 暂停中 或 部分下载未启动（含重启后恢复），圆形进度，点击继续 -->
                        <div class="download-progress clickable"
                             @click.stop="handleDownloadClick(book)"
                             v-else-if="viewMode(book.bookId) === 'paused'"
                             title="点击继续下载"
                        >
                            <el-progress
                                type="circle"
                                :width="30"
                                :stroke-width="3"
                                :percentage="downloadStatus[book.bookId]?.progress || 0"
                            />
                        </div>

                        <!-- done: 下载完成，绿色对勾 -->
                        <div class="download-complete"
                             v-else-if="viewMode(book.bookId) === 'done'"
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
import {onMounted, onUnmounted, ref, computed, nextTick} from "vue";
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
import log from "../../utils/log";
import {
    downloadBook as downloadBookApi,
    resumeBookDownload,
    pauseBookDownload,
    cancelBookDownload,
    getDownloadProgress,
    onDownloadProgress,
    onDownloadSessionExpired,
    BookInfo
} from "../../apis/bookDownload.ts";

const bookList = ref(new Array<FavoriteBookInfo>());
const page = ref(1);
const pageSize = ref(12);
const pagerCount = computed(() => favoriteGridConfig.value.pagerCount);
const totalPage = ref(1);
const booksContainer = ref<HTMLElement | null>(null);

// 获取每页数量（基于实际容器高度）
function getPageSize(): number {
    const container = booksContainer.value;
    const config = favoriteGridConfig.value;
    
    // 如果没有 cardHeight 配置（桌面端），直接返回配置的 pageSize
    if (!config.cardHeight || config.gap === undefined) {
        return config.pageSize || 9;
    }
    
    // 如果容器还没有准备好，返回当前的 pageSize（不改变）
    if (!container) {
        return pageSize.value;
    }
    
    const containerHeight = container.offsetHeight;
    
    // 如果容器高度为 0 或太小，返回当前的 pageSize（不改变）
    if (containerHeight <= 0) {
        return pageSize.value;
    }
    
    const cardTotalHeight = config.cardHeight + config.gap;
    // 减去 2px 安全边距，确保不会出现滚动
    const calculated = Math.floor((containerHeight - 2) / cardTotalHeight);
    const result = Math.max(config.minPageSize || 1, Math.min(calculated, config.maxPageSize || 6));
    
    return result;
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

// 下载状态：以"进度百分比 + 是否正在下载 + 是否暂停"三个维度派生视图状态
// （去除历史混杂的 downloaded/hasPartialDownload，唯一真源是 progress + downloading + paused）
interface DownloadStatus {
    /** 是否有活跃后台任务（前端本地维护，刷新后默认 false） */
    downloading: boolean;
    /** 是否处于暂停状态（与 downloading=true 配合表示后台任务已暂停；刷新后默认 false） */
    paused: boolean;
    /** 下载完成百分比 0~100 */
    progress: number;
}

const downloadStatus = ref<Record<number, DownloadStatus>>({});

/**
 * 统一的下载视图状态机：
 *   - 'done'    : progress >= 100，显示绿色对勾
 *   - 'loading' : downloading && !paused，显示圆形进度，点击暂停
 *   - 'paused'  : 其它任意有进度的状态（含暂停中、关掉程序后的部分进度），显示圆形进度，点击继续
 *   - 'idle'    : progress=0 且未下载，显示下载图标
 * 简化前模板里有 4 个互相打架的 v-else-if，现在所有展示分支都由此函数推导。
 */
type ViewMode = 'idle' | 'loading' | 'paused' | 'done';
function viewMode(bookId: number): ViewMode {
    const s = downloadStatus.value[bookId];
    if (!s) return 'idle';
    if (s.progress >= 100) return 'done';
    if (s.downloading && !s.paused) return 'loading';
    // 只有确实下过部分页（progress>0）才显示"暂停/继续"按钮
    // progress=0 且无活跃任务 -> 归 idle，显示下载图标（修复之前未下载的书全落 paused 的 bug）
    return s.progress > 0 ? 'paused' : 'idle';
}

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
    // 使用 nextTick 确保 DOM 渲染完成后再计算 pageSize
    nextTick(() => {
        const curPageSize = getPageSize();
        if (pageSize.value !== curPageSize) {
            pageSize.value = curPageSize;
        }
    });
});
onUnmounted(() => {
    log.debug("----- FavoriteBook Unmounted ---");
    windowSizeListener.delete(onWindowSizeChange);
    // 清理所有 Tauri 事件监听器
    for (const fn of unlistenFns) {
        try { fn(); } catch(e) { log.error('[Unlisten error]', e); }
    }
    unlistenFns.length = 0;
})

function getBookList(showLoading = true) {

    if (showLoading) {
        loading.show();
    }

    getFavoriteBookListAPi(page.value, pageSize.value)
        .then(async (bookInfoList: FavoriteBookList) => {

            empty.value = (bookInfoList.totalPage == 0);

            totalPage.value = bookInfoList.totalPage;
            bookList.value = bookInfoList.content;

            for (const book of bookInfoList.content) {
                const progressInfo = await getDownloadProgress(book.bookId);

                // 仅数据库中有该书的下载记录时，根据 downloadedPages/totalPage 计算进度
                // （downloading/paused 由本地状态保留，不依赖数据库——刷新后默认无活跃任务）
                const progress = progressInfo.exists && progressInfo.totalPage > 0
                    ? Math.min(100, Math.floor((progressInfo.downloadedPages / progressInfo.totalPage) * 100))
                    : 0;

                // 已存在本地状态时仅刷新进度，保留 downloading/paused（避免覆盖实时下载状态）
                // 但只有在没有活跃下载任务时才更新进度（防止 DB 中的旧数据覆盖实时进度）
                const existing = downloadStatus.value[book.bookId];
                if (existing) {
                    if (!existing.downloading) {
                        downloadStatus.value[book.bookId] = {
                            downloading: existing.downloading,
                            paused: existing.paused,
                            progress,
                        };
                    }
                } else {
                    downloadStatus.value[book.bookId] = {
                        downloading: false,
                        paused: false,
                        progress,
                    };
                }
            }
        })
        .finally(() => {
            if (showLoading) {
                loading.hide();
            }
        });
}

/**
 * 点击下载区按钮的统一分派：
 *   - loading: 调 pauseBookDownload 并置 paused=true
 *   - paused : 若后台任务仍在（downloading=true，真暂停），调 resumeBookDownload 并置 paused=false；
 *              若 task 已不在（重启后/部分进度未启动），调 downloadBookApi 重启下载（后端会自动跳过已下载页）
 *   - idle    : 启动新下载
 */
async function handleDownloadClick(book: FavoriteBookInfo) {
    const mode = viewMode(book.bookId);
    const prevProgress = downloadStatus.value[book.bookId]?.progress || 0;

    if (mode === 'loading') {
        // 下载中 -> 暂停
        try {
            await pauseBookDownload(book.bookId);
            const s = downloadStatus.value[book.bookId];
            if (s) {
                downloadStatus.value[book.bookId] = { ...s, paused: true };
            }
            popSuccess('已暂停');
        } catch (e) {
            log.error('[Download] pause failed:', book.bookId, e);
            popErr('暂停失败');
        }
        return;
    }

    if (mode === 'paused') {
        const s = downloadStatus.value[book.bookId];
        // 真暂停（后台 task 仍在）走 resume；其它"部分进度未启动"场景走重启下载
        const useResume = !!s?.downloading;

        // 先乐观标记为下载中
        downloadStatus.value[book.bookId] = {
            downloading: true,
            paused: false,
            progress: prevProgress,
        };

        try {
            if (useResume) {
                await resumeBookDownload(book.bookId);
                popSuccess('继续下载');
            } else {
                const result = await downloadBookApi(toBookInfo(book, prevProgress));
                if (result.success) {
                    popSuccess('继续下载');
                } else {
                    // 后端返回 success: false，可能是书籍已在下载中或所有页面已下载
                    // 检查是否所有页面已下载（通过查询下载进度）
                    const progressInfo = await getDownloadProgress(book.bookId);
                    if (progressInfo.exists && progressInfo.totalPage > 0) {
                        const progress = Math.min(100, Math.floor((progressInfo.downloadedPages / progressInfo.totalPage) * 100));
                        downloadStatus.value[book.bookId] = {
                            downloading: false,
                            paused: false,
                            progress,
                        };
                        if (progress >= 100) {
                            popSuccess('书籍已下载完成');
                        } else {
                            popErr('书籍可能正在下载中，请稍后重试');
                        }
                    } else {
                        downloadStatus.value[book.bookId] = {
                            downloading: false,
                            paused: false,
                            progress: prevProgress,
                        };
                        popErr('继续下载失败：无法启动下载任务');
                    }
                }
            }
        } catch (e) {
            // 失败回滚
            downloadStatus.value[book.bookId] = {
                downloading: false,
                paused: false,
                progress: prevProgress,
            };
            log.error('[Download] resume/restart failed:', book.bookId, e);
            popErr('继续下载失败: ' + (typeof e === 'string' ? e : JSON.stringify(e)));
        }
        return;
    }

    // idle -> 全新下载
    await startDownload(book, prevProgress);
}

/**
 * 启动一次全新下载（progress 从 0 起）。
 */
async function startDownload(book: FavoriteBookInfo, prevProgress: number = 0) {
    log.info('[Download] startDownload:', book.bookId, book.bookName);

    // 若已有活跃任务（防重复）
    const cur = downloadStatus.value[book.bookId];
    if (cur?.downloading && !cur.paused) {
        return;
    }

    downloadStatus.value[book.bookId] = {
        downloading: true,
        paused: false,
        progress: prevProgress,
    };

    try {
        const result = await downloadBookApi(toBookInfo(book, prevProgress));
        if (result.success) {
            popSuccess('下载已开始');
        } else {
            // 后端返回 success: false，可能是书籍已在下载中或所有页面已下载
            // 检查是否所有页面已下载（通过查询下载进度）
            const progressInfo = await getDownloadProgress(book.bookId);
            if (progressInfo.exists && progressInfo.totalPage > 0) {
                const progress = Math.min(100, Math.floor((progressInfo.downloadedPages / progressInfo.totalPage) * 100));
                downloadStatus.value[book.bookId] = {
                    downloading: false,
                    paused: false,
                    progress,
                };
                if (progress >= 100) {
                    popSuccess('书籍已下载完成');
                } else {
                    popErr('书籍可能正在下载中，请稍后重试');
                }
            } else {
                downloadStatus.value[book.bookId] = {
                    downloading: false,
                    paused: false,
                    progress: prevProgress,
                };
                popErr('下载失败：无法启动下载任务');
            }
        }
    } catch (e) {
        downloadStatus.value[book.bookId] = {
            downloading: false,
            paused: false,
            progress: prevProgress,
        };
        log.error('[Download] startDownload failed:', book.bookId, e);
        popErr('下载失败: ' + (typeof e === 'string' ? e : JSON.stringify(e)));
    }
}

/** 把收藏书信息转成下载接口需要的 BookInfo */
function toBookInfo(book: FavoriteBookInfo, prevProgress: number): BookInfo {
    return {
        bookId: book.bookId,
        bookName: book.bookName,
        totalPage: book.totalPage,
        coverPic: book.coverPic,
        bigCoverPic: book.bigCoverPic,
        tagId: book.tagId,
    } as BookInfo;
}

const unlistenFns: (() => void)[] = [];

onDownloadProgress((progress) => {
    log.debug('[Download] onDownloadProgress:', progress);
    if (!downloadStatus.value[progress.bookId]) {
        downloadStatus.value[progress.bookId] = {
            downloading: false,
            paused: false,
            progress: 0,
        };
    }

    const percentage = progress.totalPage > 0
        ? Math.min(100, Math.floor((progress.downloadedPages / progress.totalPage) * 100))
        : 0;

    // 整体替换整个对象以保证 Vue 3 响应式正确触发视图更新
    const current = downloadStatus.value[progress.bookId];
    downloadStatus.value[progress.bookId] = {
        downloading: current.downloading,
        paused: current.paused,
        progress: percentage,
    };

    // 下载完成：清理任务状态，标记为已完成（progress=100 -> viewMode='done'）
    if (progress.downloadedPages >= progress.totalPage) {
        downloadStatus.value[progress.bookId] = {
            downloading: false,
            paused: false,
            progress: 100,
        };
        popSuccess('下载完成');
    }
}).then(unlisten => {
    unlistenFns.push(unlisten);
});

// 后端检测到 cookie 失效（HTTP 401/403 或业务 code=100）时通知前端重登
onDownloadSessionExpired((payload) => {
    log.warn('[Download] session expired:', payload);
    const prevProgress = downloadStatus.value[payload.bookId]?.progress || 0;
    downloadStatus.value[payload.bookId] = {
        downloading: false,
        paused: false,
        progress: prevProgress,
    };
    popErr('登录已失效，请重新登录后再下载');
    router.push({ name: 'Login' });
}).then(unlisten => {
    unlistenFns.push(unlisten);
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

    log.debug(`pageIdx = ${pageIdx}, totalPage = ${totalPage.value}, page = ${page.value}`);

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
    log.debug("--- FavoriteBook Page Enter ----");

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

    getBookList(false);
}

function leave() {
    log.debug("--- FavoriteBook Page Leave ----");
    hotkeys.deleteScope('favorite');
}

defineExpose({
    'enter': enter,
    'leave': leave,
})

</script>


