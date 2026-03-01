<template>

  <div class="reading_history">

    <el-calendar ref="calendar"
                 v-model="calendarDate"
    >
      <template #header="{ date }">
        <div>{{ date }}</div>

        <div class="header_content">
          <div>月阅读时间: {{ secondsToTimeStr(monthReadTime) }}</div>
          <div>月阅读页数: {{ monthReadPage }}</div>
        </div>

        <el-button-group>
          <el-button size="small" @click="selectDate('prev-month')">
            上个月
          </el-button>
          <el-button size="small" @click="selectDate('today')">
            今天
          </el-button>
          <el-button size="small" @click="selectDate('next-month')">
            下个月
          </el-button>
        </el-button-group>
      </template>

      <template #date-cell="{ data }">

        <div v-if="data.type == 'current-month'"
             @click.prevent.capture="clickDay(data.day)"
        >

          <p :class="data.isSelected ? 'is-selected' : ''">
            {{ parseInt(data.day.split('-')[2]) }}
          </p>

          <div v-if="hasDayContent(data.day)" class="day_content">
            <div>
              <p>阅读时间: </p>
              <p>{{ secondsToTimeStr(getDayReadTime(data.day)) }}</p>
            </div>
            <div>
              <p>阅读页数: </p>
              <p>{{ getDayReadPage(data.day) }}页</p>
            </div>
          </div>

          <div v-else>

          </div>
        </div>

        <div v-else>

        </div>

      </template>
    </el-calendar>


    <div v-show="showBookList"
         class="day_book_list"
    >

      <div class="book"
           :content="book.bookName"
           v-for="book in bookHistoryList"
           :key="book.bookName"
      >

        <div class="cover">
          <img :src="addHost(book.coverPic)" :alt="book.bookName"/>
        </div>

        <div class="detail">

          <p class="name">{{ book.bookName }}</p>

          <div class="item">
            <p>阅读时间:</p>
            <p>{{ secondsToTimeStr(book.readingCost) }}</p>
          </div>

          <div class="item">
            <p>阅读进度:</p>
            <p>{{ book.startPage }}页 - {{ book.endPage }}页</p>
          </div>

          <div class="item">
            <p>阅读页数:</p>
            <p>{{ book.endPage - book.startPage }} 页</p>
          </div>

        </div>

      </div>

    </div>

    <div class="mask"
         v-show="showBookList"
         @click="showBookList = false"
    />

  </div>


</template>


<script setup lang="ts">

import {BookHistory, listReadingHistory, ReadingHistoryItem} from "../../apis/progress.ts";
import {startOfMonth, endOfMonth, format} from 'date-fns';
import {ref, watch} from "vue";
import {loadingStore} from "../../store/loading.ts";
import type {CalendarDateType, CalendarInstance} from 'element-plus';
import {secondsToTimeStr} from "../../utils/timeUtil.ts";
import {addHost} from "../../apis/request.ts";
import {StarFilled} from "@element-plus/icons-vue";
import {popErr} from "../../utils/message.ts";

const dateFormat = 'yyyy-MM-dd';

interface DayReadingHistory {
  dayReadTime: number
  dayReadPage: number
  bookList: BookHistory[]
}

const dayMap = ref(new Map<string, DayReadingHistory>());

// 本月阅读时间
const monthReadTime = ref(0);
// 本月阅读页数
const monthReadPage = ref(0);
const loading = loadingStore();
// 日历组件日期数据
const calendarDate = ref(new Date());
// 显示某一天的阅读书籍列表
const showBookList = ref(false);
// 某一天的阅读书籍列表
const bookHistoryList = ref(new Array<BookHistory>());

getReadHistory(calendarDate.value);
watch(calendarDate, (newVal, oldVal) => {
  if (newVal.getMonth() == oldVal.getMonth()) {
    return;
  }
  getReadHistory(newVal);
});


/**
 * 获取指定时间区间内的阅读历史记录
 *
 * @param date 当月日期
 */
function getReadHistory(date: Date) {

  console.log(`------ getReadHistory ${date} ------`);

  loading.show();
  const firstDay = format(startOfMonth(date), dateFormat);
  const lastDay = format(endOfMonth(date), dateFormat);
  // 获取本月历史阅读记录
  listReadingHistory(firstDay, lastDay)
      .then(res => {
        let mReadTime = 0;
        let mReadPage = 0;

        for (let item of res) {
          let bookList = item.bookList;

          let dayReadTime = 0;
          let dayReadPage = 0;

          for (let book of bookList) {
            dayReadTime += book.readingCost;
            dayReadPage += book.endPage - book.startPage;
          }
          mReadTime += dayReadTime;
          mReadPage += dayReadPage;

          dayMap.value.set(item.dayStr, {
            dayReadPage,
            dayReadTime,
            bookList
          });
        }

        monthReadTime.value = mReadTime;
        monthReadPage.value = mReadPage;
      })
      .catch(err => {
        console.log(err);
        popErr("获取阅读历史记录失败!");
      })
      .finally(() => {
        loading.hide();
      });
}


function hasDayContent(dayStr: string): boolean {
  let dayInfo = dayMap.value.get(dayStr);
  if (dayInfo == null) {
    return false;
  }
  return dayInfo.dayReadPage + dayInfo.dayReadTime != 0;
}

/**
 * 获取某一天的阅读时间, 单位 s
 *
 * @param dayStr 日期字符串
 */
function getDayReadTime(dayStr: string): number {
  let dayInfo = dayMap.value.get(dayStr);
  if (dayInfo == null) {
    return 0;
  }
  return dayInfo.dayReadTime;
}

/**
 * 获取某一天的阅读页数
 *
 * @param dayStr 日期字符串
 */
function getDayReadPage(dayStr: string): number {
  let dayInfo = dayMap.value.get(dayStr);
  if (dayInfo == null) {
    return 0;
  }
  return dayInfo.dayReadPage;
}

const calendar = ref<CalendarInstance>()

/**
 * 修改日历的数据
 *
 * @param val
 */
const selectDate = (val: CalendarDateType) => {
  if (!calendar.value) {
    return
  }
  calendar.value.selectDate(val);
}


function clickDay(day: string) {
  const dayVal = dayMap.value.get(day);
  if (dayVal == null
      || dayVal.bookList == null
      || dayVal.bookList.length == 0) {
    return;
  }

  bookHistoryList.value = dayVal.bookList;
  showBookList.value = true;
}

</script>
<style scoped lang="less" src="./ReadingHistory.less"/>
