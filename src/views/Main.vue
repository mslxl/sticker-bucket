<script setup lang="ts">
import { library } from '@fortawesome/fontawesome-svg-core'
import { faAdd, faPen, faX, faEllipsisV } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'


import MDivider from '../components/basic/Divider.vue'
import MButton from '../components/basic/Button.vue'
import MTitleBar from '../components/basic/TitleBar.vue'

import MMemeView from '../components/MemeView.vue'
import { onMounted, reactive } from 'vue'
import { getMemeByPage } from '../scripts/rs/meme'

library.add(faAdd, faPen, faX, faEllipsisV)

interface Meme{
  id: number,
  content: string,
  extraData: string,
  summary: string,
  desc: string
}

const memes = reactive<Meme[]>([])

onMounted(async () => {
  const data = await getMemeByPage(0)
  memes.push(...data)
})

</script>
<template lang="pug">
m-title-bar(title="All")
  m-button.btn-item.btn-add(@click="$router.push({name: 'meme.add'})")
    font-awesome-icon(icon="fa-solid fa-add")
    span Add
  m-divider(:vertical="true" :dark="true")
  m-button.btn-item.btn-edit
    font-awesome-icon(icon="fa-solid fa-pen")
    span Edit
  m-button.btn-item.btn-remove
    font-awesome-icon(icon="fa-solid fa-x")
    span Remove
  m-button.btn-item.btn-more 
    font-awesome-icon(icon="fa-solid fa-ellipsis-vertical")
.meme-list
  .content
    m-meme-view.meme-item(v-for="item in memes" :summary="item.summary" :image-id="item.content" :key="item.id")

</template>


<style scoped lang="scss">
.meme-list{
  height: 100%;
  width: 100%;
}
.meme-list .content {
  position: relative;
  padding: 24px;
  overflow-y: auto;
  overflow-x: hidden;
  width: 100%;
  height: 100%;
  display: grid;
  gap: 12px;

  grid-auto-rows: min-content;

  @media screen and (min-width: 240px) {
    grid-template-columns: 1fr;
  }

  @media screen and (min-width: 480px) {
    grid-template-columns: 1fr 1fr;
  }

  @media screen and (min-width: 720px) {
    grid-template-columns: 1fr 1fr 1fr;
  }

  @media screen and (min-width: 960px) {
    grid-template-columns: 1fr 1fr 1fr 1fr;
  }

  @media screen and (min-width: 1200px) {
    grid-template-columns: 1fr 1fr 1fr 1fr 1fr;
  }
}



.btn-item {
  overflow: hidden;
  white-space: nowrap;
  line-height: 20px;

  span {
    padding: 0 8px 0 8px;
  }
}</style>