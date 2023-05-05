<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRoute } from 'vue-router'
import { getMemeByID, getImageRealPath, getTagByMemeID } from '../scripts/rs/db'

import { library } from '@fortawesome/fontawesome-svg-core'
import { faPenToSquare } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import { ElCard, ElButton } from 'element-plus'
import { ElTag, ElSpace } from 'element-plus'
import MTitlebar from '../components/basic/TitleBar.vue'

import { convertFileSrc } from '@tauri-apps/api/tauri'

library.add(faPenToSquare)

interface Meme {
  id: number,
  content: string,
  extraData: string,
  summary: string,
  desc: string
}

const route = useRoute()
const imageId = Number(route.params['id'])

const meme = ref<Meme>()
const tags = reactive<Map<string, string[]>>(new Map())

const imageURL = ref('')

async function loadData() {
  tags.clear()
  meme.value = await getMemeByID(imageId)
  imageURL.value = convertFileSrc(await getImageRealPath(meme.value.content))
  for (let tag of await getTagByMemeID(meme.value.id)) {
    if (!tags.has(tag.namespace)) tags.set(tag.namespace, [])
    tags.get(tag.namespace)!.push(tag.value)
  }
}
onMounted(() => {
  loadData()
})
</script>

<template lang="pug">
m-titlebar(:title="meme?.summary || 'Loading'" :back="true")
  el-button.btn-item(
    type=""
    text)
    font-awesome-icon(icon="fa-solid fa-pen-to-square")
    span Edit

.panel
  el-card.image__overlay
    img(:src="imageURL")
  .info
    span.name Summary
    span.value {{ meme?.summary }}
    template(v-if="meme?.desc")
      span.name Description
      span.value {{ meme?.desc }}
  .tag-panel
    template(v-for="space in tags.entries()")
      el-tag(type="info" effect="dark") {{ space[0] }}
      el-space
        el-tag.selectable(
          v-for="value in space[1]"
          :key="value"
          type="info"
          effect="plain") {{ value }}
      
</template>

<style scoped lang="scss">
.panel {
  overflow-y: auto;
  width: 100%;
  height: 100%;

  display: flex;
  justify-content: start;
  align-items: center;
  flex-direction: column;

  .image__overlay {
    width: 50%;
    height: 50%;
    padding: 12px;
    display: flex;
    justify-content: center;
    align-items: center;

    img {
      display: block;
      max-width: 100%;
      max-height: 100%;
    }
  }

  .info {
    margin-top: 12px;

    span {
      padding: 8px 4px 8px 4px;
      border: 1px solid var(--primary-border-color);
    }
  }
}

.info {
  display: grid;
  grid-template-columns: auto 1fr;
}

.tag-panel {
  margin: 12px;
  padding: 12px;
  display: grid;
  width: 100%;
  gap: 8px;
  grid-template-columns: auto 1fr;

}

.btn-item {
  span {
    padding: 0 8px 0 8px;
  }
}
</style>