<script setup lang="ts">
import { library } from '@fortawesome/fontawesome-svg-core'
import { faAdd, faPen, faX, faEllipsisV } from '@fortawesome/free-solid-svg-icons'
import { faStar, faPaperPlane } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'


import { ElButton, ElButtonGroup } from 'element-plus'
import { ElInput } from 'element-plus'
import { ElDropdownItem, ElDropdown, ElDropdownMenu } from 'element-plus'
import MTitleBar from '../components/basic/TitleBar.vue'

import MMemeView from '../components/MemeView.vue'
import { reactive, ref } from 'vue'
import { getMemeByPage, updateFavById } from '../scripts/rs/db'
import { useRoute } from 'vue-router'

library.add(faAdd, faPen, faX, faEllipsisV)
library.add(faStar, faPaperPlane)

const route = useRoute()

const onlyFav = route.name == 'fav'

interface Meme {
  id: number,
  content: string,
  extraData: string,
  summary: string,
  desc: string,
  fav: boolean,
  trash: boolean
}

const memes = ref<Meme[]>([])
const memeListDiv = ref<HTMLDivElement>()
const searchStatement = ref('')

let currentPage = 0

async function updateSearchStmt() {
  currentPage = 0
  memes.value.splice(0, memes.value.length)
  loadNextPage()
}

function loadNextPage() {
  getMemeByPage(searchStatement.value, currentPage, onlyFav ? "OnlyFav" : "Normal").then(data => {
    if (data.length > 0) {
      currentPage++;
      memes.value.push(...data)
    }
  }).catch(e => {
    console.error(e)
  })
}

function toggleFav(index: number){
  memes.value[index].fav = !memes.value[index].fav
  updateFavById(memes.value[index].id, memes.value[index].fav)
}

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
    template(#default v-if="!onlyFav")
      el-dropdown(
        type=""
        split-button
        text
        @click="$router.push({name: 'meme.add'})")
        font-awesome-icon(icon="fa-solid fa-add")
        span Add
        template(#dropdown)
          el-dropdown-menu
            el-dropdown-item(@click="$router.push({ name: 'meme.add.bulk' })") Bulk Add
      el-button.btn-item.btn-more(
        type=""
        text)
        font-awesome-icon(icon="fa-solid fa-ellipsis-vertical")
  ul.meme-list(ref="memeListDiv" v-infinite-scroll="loadNextPage" :infinite-scroll-distance="60")
    li(v-for="(item, index) in memes")
      m-meme-view.meme-item(
        :summary="item.summary" 
        :image-id="item.content" 
        :key="item.id" 
        @meme-click="$router.push({name: 'meme.view', params: {id: item.id}})")
        div(style="width: 100%; display: flex; justify-content: right;")
          el-button-group.meme-btn-bar
            el-button(type="" size="large" text)
              font-awesome-icon(icon="fa-solid fa-paper-plane")
            el-button(type="" size="large" text @click="$router.push({name: 'meme.info_edit', params: {id : item.id}})") 
              font-awesome-icon(icon="fa-solid fa-pen")
            el-button(type="" size="large" @click="toggleFav(index)" text) 
              font-awesome-icon(icon="fa-solid fa-star" :class="{star: item.fav}")

</template>


<style scoped lang="scss">
.star{
  color: orange;
}
.meme-btn-bar {
  padding: 12px;
}

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