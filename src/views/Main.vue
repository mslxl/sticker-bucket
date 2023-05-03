<script setup lang="ts">
import { library } from '@fortawesome/fontawesome-svg-core'
import { faAdd, faPen, faX, faEllipsisV } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'


import { ElButton, ElButtonGroup, ElInfiniteScroll } from 'element-plus'
import { ElInput } from 'element-plus'
import MTitleBar from '../components/basic/TitleBar.vue'

import MMemeView from '../components/MemeView.vue'
import { onMounted, reactive, ref } from 'vue'
import { getMemeByPage } from '../scripts/rs/db'

library.add(faAdd, faPen, faX, faEllipsisV)

interface Meme {
  id: number,
  content: string,
  extraData: string,
  summary: string,
  desc: string
}

const memes = reactive<Meme[]>([])
const memeListDiv = ref<HTMLDivElement>()
const searchStatement = ref('')

let currentPage = 0

async function updateSearchStmt() {
  currentPage = 0
  memes.splice(0, memes.length)
  loadNextPage()
}

function loadNextPage() {
  getMemeByPage(searchStatement.value, currentPage).then(data => {
    if (data.length > 0) {
      currentPage++;
      memes.push(...data)
    }
  }).catch(e => {
    console.error(e)
  })
}

// onMounted(async () => {
//   await updateSearchStmt()
// })

</script>
<template lang="pug">
.main
  m-title-bar(title="All")
    template(#content)
      el-input(
        style="padding-left: 24px; padding-right: 24px;" 
        placeholder="Search" 
        v-model="searchStatement" 
        @input="updateSearchStmt")
    template(#default)
      el-button-group
        el-button.btn-item.btn-add(
          type=""
          text
          @click="$router.push({name: 'meme.add'})")
          font-awesome-icon(icon="fa-solid fa-add")
          span Add
        el-button.btn-item.btn-remove(
          type=""
          text)
          font-awesome-icon(icon="fa-solid fa-x")
          span Remove
        el-button.btn-item.btn-more(
          type=""
          text)
          font-awesome-icon(icon="fa-solid fa-ellipsis-vertical")
  ul.meme-list(ref="memeListDiv" v-infinite-scroll="loadNextPage" :infinite-scroll-distance="60")
    li(v-for="item in memes" )
      m-meme-view.meme-item(
        :summary="item.summary" 
        :image-id="item.content" 
        :key="item.id" 
        @click="$router.push({name: 'meme.view', params: {id: item.id}})")

</template>


<style scoped lang="scss">
.main {
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
}

ul {
  list-style-type: none;

  li {
    display: block;
  }

  li>.meme-item {
    width: 100%;
    height: 100%;
  }
}

.meme-list {
  position: relative;
  padding: 24px;
  overflow-y: auto;
  overflow-x: hidden;
  width: 100%;
  height: 100%;
  display: grid;
  gap: 12px;
  flex-grow: 1;

  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
}


.btn-item {
  overflow: hidden;
  white-space: nowrap;
  line-height: 20px;

  span {
    padding: 0 8px 0 8px;
  }
}
</style>