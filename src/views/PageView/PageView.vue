<template>

  <div class="page_view">

    <Contents ref="contents"
              :book-id="bookId"
              :book-name="bookInfo.bookName"
              @skip-chapter="getPageHtml"
              @skip-cover-page="getPageHtml(PageCache.COVER_PAGE)"
              @open="showContents = true"
              @close="showContents = false"
    />

    <div class="page_container"
         @touchstart.stop="touchControl.touchstart"
         @touchmove.stop="touchControl.touchmove"
         @touchend.stop="touchControl.touchend"
         @click="toggleFooter"
         ref="pageContainer"
         :class="{
                    'show_contents': showContents
                }"
    >


      <div class="book_cover"
           :style="{backgroundImage: 'url(' + addHost(bookInfo.bigCoverPic) + ')'}"
           v-if="curPageItem.page === PageCache.COVER_PAGE">
      </div>

      <template v-else>


        <div class="page_content"

        >

          <div class="page_indicator">
            <p class="page_title">{{ curPageItem.title ?? '' }}</p>
            <div class="page_number">

              <input
                  type="number"
                  :value="curPageItem.page"
                  maxlength="4"
                  min="1"
                  :max="totalPage"
                  @change="getPageHtml($event.target.value)"/>

              <p>/{{ totalPage }} 页</p>
            </div>
          </div>

          <div class="raw_page" ref="rawPage" id="raw_page"
               @click="rawPageClick"
               v-html="curPageItem.content"
               :style="{
                            'scale': 1 +  scale.current / 100,
                            'margin-top': scale.current - 50 * (1 -  scale.current / 20) + 'px'
                         }"
          ></div>


          <div class="page_bottom"
               v-if="showClock">
            <Clock/>
          </div>

        </div>


      </template>


    </div>


    <el-image-viewer
        v-if="imageViewer"
        :url-list="previewList"
        :hide-on-click-modal="true"
        @close="imageViewerClose"
    />
  </div>


  <ViewFooterBar v-show="showFooter"
                 @open-contents="openContents"
  />

  <el-dialog
      class="page_confirm"
      v-model="pageConfirm"
      :show-close="false"
      center
      width="500"
  >
    <span>本地进度为 {{ pageConfirmLocal }} 页, 远程进度为 {{ pageConfirmRemote }} 页</span>
    <span> 请选择要保留的进度 </span>
    <template #footer>
      <div class="page_confirm_footer">
        <el-button @click="pageConfirm = false; updateReadProgress(bookId,pageConfirmLocal)">本地</el-button>
        <el-button @click="pageConfirm = false; getPageHtml(pageConfirmRemote)">远程</el-button>
      </div>
    </template>
  </el-dialog>


</template>

<style src="./PageView.less" lang="less"/>

<script setup lang="ts">
import {onBeforeUnmount, onMounted, ref, useTemplateRef} from 'vue';
import {useRoute, useRouter} from 'vue-router';
import Contents from './Contents.vue';
import {BookInfo, PageItem} from "../../model/pageModel";
import {PageCache} from "./pageCache";
import {getBookInfo} from "../../apis/book";
import {getLocalStorageInt, setLocalStorage} from "../../utils/localStorageUtil";
import {addPopover} from "../../service/bookPageFormatter";
import hotkeys from 'hotkeys-js';
import {TouchControl} from "../../service/touchControl";
import {PageType, pageTypeStore} from "../../store/pageType";
import {ScreenResizeListener} from "./fullScreenListener";
import Clock from "../../components/Clock.vue";
import {scaleStore} from "../../store/scale";
import ViewFooterBar from "../../components/ViewFooterBar.vue";
import {getRemotePage} from "../../apis/userRemotePage";
import {addHost} from "../../apis/request.ts";
import {recordReadingProgress, updateReadProgress} from "./pageView.ts";
import {getDownloadProgress} from "../../apis/bookDownload.ts";

// 是否显示底部时间
const showClock = ref(false);
// 当前显示页面
const pageType = pageTypeStore();
// 书页方法倍数
const scale = scaleStore();
// 显示底部控制框
const showFooter = ref(true);
// 目录是否显示
const showContents = ref(false);

// 判断是否全屏
const screeResize = new ScreenResizeListener();
screeResize.addListener((full: boolean) => {
  showClock.value = full;
});

onMounted(() => {
  pageType.current = PageType.PAGE_VIEW;
});


let bookId = 0;
const totalPage = ref(0);
// 书籍是否是本地下载的
const isLocalBook = ref(false);

// 当前显示的页面数据
const curPageItem = ref<PageItem>(PageCache.BLANK_PAGE_ITEM);

const bookInfo = ref<BookInfo>({
  bigCoverPic: "",
  tagId: 0,
  bookName: '',
  totalPage: 1,
  coverPic: '',
  bookId: 0,
  favorite: false,
});

// 图片预览列表
const previewList = ref(new Array<string>());
// 图片预览
const imageViewer = ref(false);
const route = useRoute();
const router = useRouter();

const bookIdStr = <string>route.query.bookId;
// 书籍是否被收藏
const isFavorite = route.query.favorite == "true";

// 找不到有效的 bookId 跳回到书籍列表
if (bookIdStr == null || isNaN(parseInt(bookIdStr))) {
  router.push({
        name: "Home"
      }
  );
}
bookId = parseInt(bookIdStr);

// 页面缓存
let pageCache: PageCache | null = null;

// 获取书籍信息并初始化
getBookInfo(bookId).then(async (resBookInfo: BookInfo) => {
  bookInfo.value = resBookInfo;
  // 设置一下 bookId, 接口没有返回 bookId
  bookInfo.value.bookId = bookId;
  // 将 totalPage
  totalPage.value = resBookInfo.totalPage;
  // 检查书籍是否已下载（只要在 book 表中存在就使用本地模式）
  const downloadInfo = await getDownloadProgress(bookId);
  const useLocal = downloadInfo.exists;
  isLocalBook.value = useLocal;

  // 创建页面缓存，从本地读取已下载的页面
  pageCache = new PageCache(bookId, useLocal);
  pageCache.setTotalPage(totalPage.value);
  
  // 初始化页面
  initPage(bookId);
});

// 快速选择页面中的图片
function imageSelect(e: any) {
  e.preventDefault();

  if (imageViewer.value) {
    imageViewer.value = false;
    return;
  }

  let imageList = document.querySelectorAll("#raw_page img");
  let srcList = new Array<string>();
  // @ts-ignore
  for (let img of imageList) {
    let src = img.attributes.getNamedItem("src");
    if (src instanceof Attr && src.value != null) {
      // 确保 URL 是完整的
      let imageUrl = src.value;
      if (!imageUrl.includes("://")) {
        // 如果不是完整 URL，使用 addHost 处理
        imageUrl = addHost(imageUrl);
      }
      srcList.push(imageUrl);
    }
  }

  if (srcList.length != 0) {
    previewList.value = srcList;
    imageViewer.value = true;
  }
}

hotkeys('left, a, s, page up', 'page-view', prePage);
hotkeys('right, f, d, page down', 'page-view', nextPage);
hotkeys('w', 'page-view', imageSelect);
hotkeys.setScope('page-view');
// console.log(hotkeys.getAllKeyCodes());
onBeforeUnmount(() => {
  hotkeys.deleteScope('page-view');
});

/**
 * 关闭图片预览
 */
function imageViewerClose() {
  imageViewer.value = false;
}

/**
 * 点击图片后开启预览
 * @param event
 */
function rawPageClick(event: any) {
  const target = event.target;
  if (target.src != undefined) {
    let imageUrl = target.src;
    // 确保 URL 是完整的
    if (!imageUrl.includes("://")) {
      // 如果不是完整 URL，使用 addHost 处理
      imageUrl = addHost(imageUrl);
    }
    previewList.value = [imageUrl];
    imageViewer.value = true;
  }
}

/**
 * 下一页
 */
function nextPage() {
  if (imageViewer.value) {
    return;
  }
  getPageHtml(curPageItem.value.page + 1);
}

/**
 * 上一页
 */
function prePage() {
  if (imageViewer.value) {
    return;
  }
  getPageHtml(curPageItem.value.page - 1);
}

const pageContainer = ref();
const rawPage = ref();

// 添加书页弹出的注解
addPopover(curPageItem);

let mounted = false;
onMounted(() => {
  mounted = true;
});


const pageConfirm = ref(false);
const pageConfirmRemote = ref(0);
const pageConfirmLocal = ref(0);

// 记录阅读进度
recordReadingProgress(bookId, curPageItem, pageConfirm, isLocalBook.value);

async function initPage(bookId: number) {
  let localPage = getLocalStorageInt(bookId, PageCache.COVER_PAGE);
  // 获取服务端书页进度
  let remotePage = await getRemotePage(bookId);

  // -1 说明没有收藏该书, 不记录该书的进度
  if (remotePage == -1) {
    getPageHtml(localPage);
    return;
  }

  // 存在远程进度, 还是先获取本地进度书页
  getPageHtml(localPage);

  if (remotePage != localPage) {
    // 服务端进度和本地进度不一致, 手动确认要保留的进度
    // 默认使用 本地进度，手动选择远程进度后切换为远程进度
    pageConfirmLocal.value = localPage;
    pageConfirmRemote.value = remotePage;
    pageConfirm.value = true;
  }
}

// 记录阅读进度
recordReadingProgress(bookId, curPageItem, pageConfirm);

function getPageHtml(curPage: number) {

  if (curPage <= PageCache.COVER_PAGE) {
    if (PageCache.COVER_PAGE != curPageItem.value.page) {
      curPageItem.value = PageCache.COVER_PAGE_ITEM;

      // 同步修改当前页所在目录
      contents?.value?.changePage(PageCache.COVER_PAGE);

      // 记录上一次获取的页面
      setLocalStorage(bookId, PageCache.COVER_PAGE);
    }
    return;
  }

  // totalPage = 0 是书籍总页数还没有获取到时
  if (totalPage.value > 0 && curPage > totalPage.value) {
    curPage = totalPage.value;
  }

  if (curPage < PageCache.FIRST_PAGE) {
    curPage = PageCache.FIRST_PAGE;
  }

  // page 没有变更
  if (curPage == curPageItem.value.page) {
    return;
  }

  // 没挂载前不能操作 DOM 元素
  if (mounted) {
    pageContainer.value.scrollTop = 0;
  }

  pageCache?.getPage(curPage).then((pageItem: PageItem | null) => {
    // 多次重复获取不操作直接返回
    if (pageItem == null) {
      return;
    }
    curPageItem.value = pageItem;

    // 同步修改当前页所在目录
    contents?.value?.changePage(pageItem.page);

    // 记录上一次获取的页面
    setLocalStorage(bookId, curPage);
  })
}

const touchControl = new TouchControl();
touchControl.onSwipeLeft(nextPage);
touchControl.onSwipeRight(prePage);

function toggleFooter() {
  showFooter.value = !showFooter.value;
}

const contents = useTemplateRef<InstanceType<typeof Contents>>("contents");

function openContents() {
  showFooter.value = false;
  contents?.value?.show();
}

</script>

