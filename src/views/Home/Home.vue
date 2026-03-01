<template>

  <div class="home">

    <div class="tabs">

      <div class="tab"
           @click="changeTab(Tab.Library)"
           :class="{
                     'active': curTab == Tab.Library
                }"
      >
        <el-icon>
          <Grid/>
        </el-icon>
        <p>书库</p>
      </div>

      <div class="tab"
           @click="changeTab(Tab.Favorite)"
           :class="{
                     'active': curTab == Tab.Favorite
                }"
      >
        <el-icon>
          <Management/>
        </el-icon>
        <p>收藏</p>
      </div>

      <div class="tab"
           @click="changeTab(Tab.ReadingHistory)"
           :class="{
                     'active': curTab == Tab.ReadingHistory
                }"
      >
        <el-icon>
          <Histogram/>
        </el-icon>
        <p>阅读记录</p>
      </div>

      <div class="tab"
           @click="changeTab(Tab.Download)"
           :class="{
                     'active': curTab == Tab.Download
                }"
      >
        <el-icon>
          <Download/>
        </el-icon>
        <p>下载</p>
      </div>


      <div class="tab"
           @click="changeTab(Tab.Config)"
           :class="{
                     'active': curTab == Tab.Config
                }"
      >
        <el-icon>
          <Tools/>
        </el-icon>
        <p>设置</p>
      </div>

    </div>


    <div class="content">

      <BookShelf ref="bookshelf_view" v-show="curTab == Tab.Library"/>

      <FavoriteBook ref="favorite_view" v-show="curTab == Tab.Favorite"/>

      <ReadingHistory ref="reading_history" v-if="curTab == Tab.ReadingHistory"/>

      <DownloadBook ref="download_view" v-show="curTab == Tab.Download"
/>

      <Config ref="config_view" v-show="curTab == Tab.Config"/>

    </div>


  </div>


</template>

<script setup lang="ts">
import BookShelf from "../BookShelf/BookShelf.vue";
import {onMounted, ref, useTemplateRef} from 'vue';
import Config from "../Config/Config.vue";
import {Grid, Histogram, Management, Tools, Download} from "@element-plus/icons-vue";
import FavoriteBook from "../FavoriteBook/FavoriteBook.vue";
import {getLocalStorageInt, setLocalStorage} from "../../utils/localStorageUtil.ts";
import ReadingHistory from "../ReadingHistory/ReadingHistory.vue";
import DownloadBook from "../DownloadBook/DownloadBook.vue";

const HOME_TAB = "home_tab";

enum Tab {
  Library,
  Favorite,
  Config,
  ReadingHistory,
  Download
}

const curTab = ref<Tab>(getLocalStorageInt(HOME_TAB, Tab.Library));

const favoriteView = useTemplateRef<InstanceType<typeof FavoriteBook>>("favorite_view");
const bookShelfView = useTemplateRef<InstanceType<typeof BookShelf>>("bookshelf_view");
const downloadView = useTemplateRef<InstanceType<typeof DownloadBook>>("download_view");

function changeTab(tab: Tab, isInit = false) {
  if (curTab.value == tab && !isInit) {
    return;
  }

  switch (curTab.value) {
    case Tab.Library: {
      bookShelfView?.value?.leave();
      break;
    }
    case Tab.Favorite: {
      favoriteView?.value?.leave();
      break;
    }
    case Tab.Download: {
      downloadView?.value?.leave();
      break;
    }
  }

  switch (tab) {
    case Tab.Library: {
      bookShelfView?.value?.enter();
      break;
    }
    case Tab.Favorite: {
      favoriteView?.value?.enter();
      break;
    }
    case Tab.Download: {
      downloadView?.value?.enter();
      break;
    }
  }



  curTab.value = tab;
  setLocalStorage(HOME_TAB, tab);
}


onMounted(() => {
  // 刷新显示页面的内容
  changeTab(curTab.value, true);
})


</script>

<style scoped lang="less" src="./Home.less"/>

