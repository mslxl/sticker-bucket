<!-- 表情信息编辑 -->

<script setup lang="ts">
import { library } from '@fortawesome/fontawesome-svg-core'
import { faCrop, faCheck } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { ElRow, ElCol, ElSpace} from 'element-plus'
import { ElImage, ElCard, ElLoading } from 'element-plus'
import { ElButton, ElMessageBox } from 'element-plus'
import { ElContainer, ElHeader, ElMain } from 'element-plus'
import Titlebar from '../components/basic/TitleBar.vue'
import MemeBasicInfoEditor from '../components/MemeBasicInfoEditor.vue'

import { getMemeByID, getTagByMemeID, getImageRealPath, updateMeme } from '../scripts/rs/db'
import { convertFileSrc } from '@tauri-apps/api/tauri'

library.add(faCrop, faCheck)

export interface Meme{
  id: number,
  content: string,
  extraData: string,
  summary: string,
  desc: string
}

export interface Tag{
  namespace: string,
  value: string
}

const imagePath = ref('')
const imageURL = computed(() => imagePath.value.trim().length > 0 ? convertFileSrc(imagePath.value): '')
const editorSummary = ref('')
const editorDesc = ref('')
const editorTags = ref<Tag[]>([])

let oldData: Meme;
let oldTags: Tag[];

const route = useRoute()
const router = useRouter()
const imageId = Number(route.params['id'])

onMounted(async () => {
  const loadingInstance = ElLoading.service({ fullscreen: true })
  let meme = await getMemeByID(imageId)
  oldData = meme;
  let tags = await getTagByMemeID(imageId)
  oldTags = tags
  editorTags.value = tags
  editorDesc.value = meme.desc
  editorSummary.value = meme.summary
  imagePath.value = await getImageRealPath(meme.content)
  loadingInstance.close()
})

async function saveChange(){
  let loadingInstance = ElLoading.service({fullscreen: true})
  try{
    await updateMeme(
      imageId,
      editorSummary.value != oldData.summary ? editorSummary.value : undefined,
      editorDesc.value != oldData.desc ? editorDesc.value : undefined,
      editorTags.value.sort() != oldTags.sort() ? editorTags.value : undefined)
  }catch(e: any){
    ElMessageBox.alert(e.message)
  }
  router.back()
  loadingInstance.close()
}

</script>

<template lang="pug">
el-container(style="height: 100%;")
  el-header
    titlebar(:back="true" title="Edit info")
      el-button(type="" text @click="saveChange") 
        el-space
          font-awesome-icon(icon="fa-solid fa-check")
          span Save
  el-main(style="height: 100%;")
    el-row(style="height: 100%; display: flex; align-items: center;" :gutter="24")
      el-col(:span="12")
        el-card.image-card
          el-image.image(
            fit="contain"
            :src="imageURL")
          el-button(type="" text)
            font-awesome-icon(icon="fa-solid fa-crop")

            

      el-col(:span="12")
        meme-basic-info-editor(
          :allow-lock="false"
          :no-delete="true"
          :delete-file="false"
          v-model:summary="editorSummary"
          v-model:description="editorDesc"
          v-model:tags="editorTags")

</template>

<style scoped lang="scss">
.image-card {

  text-align: right;

  :deep(.el-card__body) {
    width: 100%;
    height: 100%;
  }

  .image {
    width: 100%;
    height: 100%;
  }
}
</style>