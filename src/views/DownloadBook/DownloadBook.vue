<template>

    <div class="favorite">

        <div class="empty-notify"
             :class="{
                'active': empty
            }"
        >
            <el-icon>
                <Download/>
            </el-icon>
            <p>还没有下载书籍</p>
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

                    <div class="delete-download"
                         @click="deleteBook(book.bookId)"
                    >
                        <el-icon>
                            <Delete/>
                        </el-icon>

                    </div>

                    <p class="name">{{ book.bookName }}</p>

                    <!-- 阅读进度（如果有阅读记录） -->
                    <div class="item" v-if="book.readPage > 0">
                        <p>阅读进度:</p>
                        <div class="detail_item_content">
                            <p>{{ book.readPage }} / {{ book.totalPage }} 页</p>
                            <el-progress
                                :text-inside="true"
                                :stroke-width="15"
                                :format="num => `${num == 0 ? '0' : num.toFixed(2)}%`"
                                :percentage="book.readPage / book.totalPage * 100"
                            />
                        </div>
                    </div>

                    <div class="item" v-if="book.readPage > 0">
                        <p>阅读时间:</p>
                        <p>{{ readCost(book.readingCost) }}</p>
                    </div>
                    <div class="item" v-if="book.readPage > 0">
                        <p>上次阅读:</p>
                        <p>{{ book.lastRead == 0 ? '未阅读' : getLastRead(book.lastRead) }}</p>
                    </div>

                    <div class="item">
                        <p>下载状态:</p>
                        <div class="detail_item_content">
                            <p>{{ book.downloadedPages }} / {{ book.totalPage }} 页</p>
                            <el-progress
                                :text-inside="true"
                                :stroke-width="15"
                                :format="num => `${num}%`"
                                :percentage="book.progress"
                            />
                        </div>
                    </div>

                    <div class="item">
                        <p>下载时间:</p>
                        <p>{{ formatDate(book.createTime) }}</p>
                    </div>


                </div>

            </div>

        </div>


        <el-pagination
            v-model:current-page="page"
            v-model:page-size="pageSize"
            layout="prev, pager, next, jumper"
            :page-count="totalPage"
            @current-change="jumpToPage"
            hide-on-single-page
        />

    </div>

</template>

<style scoped lang="less" src="./DownloadBook.less"/>

<script setup lang="ts">
import {addHost} from "../../apis/request.ts";
import {onUnmounted, ref} from "vue";
import {useRouter} from "vue-router";
import {Download, Delete} from "@element-plus/icons-vue";
import {popErr, popSuccess} from "../../utils/message.ts";
import {loadingStore} from "../../store/loading.ts";
import hotkeys from "hotkeys-js";
import windowSizeListener from "../../service/windowSize.ts";
import {formatDistanceToNow} from 'date-fns';
import {zhCN} from 'date-fns/locale';
import {
    getDownloadedBookListByPage,
    deleteLocalBook,
    DownloadedBookInfo
} from "../../apis/bookDownload.ts";

const bookList = ref<DownloadedBookInfo[]>([]);
const page = ref(1);
const pageSize = ref(12);
const totalPage = ref(1);
const router = useRouter();
const empty = ref(false);
const loading = loadingStore();

// 监听窗口大小变化，修改 pageSize
const onWindowSizeChange = (width: number, height: number) => {
    let curPageSize = 12;
    if (height < 500) {
        curPageSize = 4;
    } else if (height <= 900) {
        curPageSize = 9;
    }
    if (pageSize.value != curPageSize) {
        pageSize.value = curPageSize;
        getBookList();
    }
};
windowSizeListener.on(onWindowSizeChange);
onUnmounted(() => {
    console.log("----- DownloadBook Unmounted ---");
    windowSizeListener.delete(onWindowSizeChange);
})

function getBookList() {
    loading.show();
    getDownloadedBookListByPage(page.value, pageSize.value)
        .then((result) => {
            bookList.value = result.content;
            totalPage.value = result.totalPage;
            empty.value = result.total === 0;
        })
        .finally(() => {
            loading.hide();
        });
}

function toBookPage(book: DownloadedBookInfo) {
    // 跳转到最后阅读的页面，如果没有阅读记录则跳转到第1页
    const targetPage = book.readPage > 0 ? book.readPage : 1;
    setTimeout(() => {
        router.push({
            name: "Read",
            query: {
                "bookId": book.bookId,
                "remotePage": targetPage,
                "local": "true"
            }
        }).then();
    }, 200)
}

function jumpToPage(pageIdx: number) {
    if (pageIdx < 1 || (totalPage.value != 0 && pageIdx > totalPage.value)) {
        return;
    }
    page.value = pageIdx;
    getBookList();
}

function deleteBook(bookId: number) {
    deleteLocalBook(bookId)
        .then(() => {
            popSuccess("删除成功");
            getBookList();
        })
        .catch(() => {
            popErr("删除失败");
        });
}

function formatDate(timestamp: number): string {
    if (!timestamp) return '未知';
    const date = new Date(timestamp * 1000);
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}

/**
 * 格式化阅读时间
 * @param minutes 分钟数
 */
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

/**
 * 获取上次阅读时间的友好显示
 * @param lastReadTime 上次阅读时间戳
 */
function getLastRead(lastReadTime: number): string {
    return formatDistanceToNow(new Date(lastReadTime),
        {
            addSuffix: true,
            locale: zhCN,
        });
}


function enter() {
    console.log("--- DownloadBook Page Enter ----");
    hotkeys('left, a, s, page up', 'download', () => jumpToPage(page.value - 1));
    hotkeys('right, f, d, page down', 'download', () => jumpToPage(page.value + 1));
    hotkeys.setScope('download');
    getBookList();
}

function leave() {
    console.log("--- DownloadBook Page Leave ----");
    hotkeys.deleteScope('download');
}

defineExpose({
    'enter': enter,
    'leave': leave,
})
</script>
