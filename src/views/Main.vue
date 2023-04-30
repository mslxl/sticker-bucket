<script setup lang="ts">
import { library } from '@fortawesome/fontawesome-svg-core'
import { faAdd, faPen, faX, faEllipsisV } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'


import MDivider from '../components/basic/Divider.vue'
import MButton from '../components/basic/Button.vue'
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

let currentPage = 0
async function loadNextPage(): Promise<boolean> {
  const data = await getMemeByPage(currentPage)
  if(data.length > 0){
    currentPage++;
    memes.push(...data)
    return true
  }
  return false
}


onMounted(async () => {
  currentPage = 0
  loadNextPage()
  memeListDiv.value!.onscroll = () => {
    let scrollTop = memeListDiv.value!.scrollTop
    let scrollHeight = memeListDiv.value!.scrollHeight
    let clientHeight = memeListDiv.value!.clientHeight

    if(clientHeight + scrollTop >= scrollHeight){
      loadNextPage()
    }
  }
})

</script>
<template lang="pug">
.main
  m-title-bar(title="All")
    m-button.btn-item.btn-add(@click="$router.push({name: 'meme.add'})")
      font-awesome-icon(icon="fa-solid fa-add")
      span Add
    m-divider(:vertical="true" :dark="true")
    m-button.btn-item.btn-remove
      font-awesome-icon(icon="fa-solid fa-x")
      span Remove
    m-button.btn-item.btn-more 
      font-awesome-icon(icon="fa-solid fa-ellipsis-vertical")
  .meme-list(ref="memeListDiv")
    m-meme-view.meme-item(
      v-for="item in memes" 
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