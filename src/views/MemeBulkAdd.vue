<script setup lang="ts">
import { library } from '@fortawesome/fontawesome-svg-core'
import { faArrowRight, faTrash } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/tauri'

import { ElLoading, ElCard, ElImage } from 'element-plus'
import { ElContainer, ElHeader, ElMain } from 'element-plus'
import { ElProgress, ElButton, ElButtonGroup} from 'element-plus'
import { ElResult, ElMessage } from 'element-plus'

import { openPicturesList } from '../scripts/rs/local'
import { addMemeToLib } from '../scripts/rs/db'
import MTitlebar from '../components/basic/TitleBar.vue'
import MemeBasicInfoEditor from '../components/MemeBasicInfoEditor.vue'

library.add(faArrowRight, faTrash)

const picPath = ref<string[]>([])
const picIdx = ref(-1)
const router = useRouter()

const basename = computed(()=>{
  if(picIdx.value == -1){
    return ""
  }else{
    let filename = picPath.value[picIdx.value]
    let regex = /.*(?:\/|\\)(.*?)$/
    let matches = regex.exec(filename)!
    return matches[1]
  }
})
const imageLocalURL = computed(() => basename.value.length > 0 ? convertFileSrc(picPath.value[picIdx.value]) : '')

onMounted(async () => {
  let loadingInstance = ElLoading.service({
    fullscreen: true
  })
  try {
    let list = await openPicturesList()
    picPath.value = list
    picIdx.value = 0
    loadingInstance.close()
  } catch (e) {
    loadingInstance.close()
    router.back()
  }
})

const editorDeleteFile = ref(true)
const editorSummary = ref('')
const editorDesc = ref('')
const editorTags = ref<{namespace: string, value: string, lock?: boolean}[]>([])

function nextPicture(){
  if(picIdx.value != picPath.value.length){
    picIdx.value = picIdx.value + 1
    editorSummary.value = ''
    editorDesc.value = ''
    editorTags.value = editorTags.value.filter(tag=>tag.lock === true)
  }
}

async function addMeme(){
  if(picIdx.value == picPath.value.length){
    return
  }
  try{
    if(editorSummary.value.trim().length == 0) {
      ElMessage.error('Summary can not be empty')
      return
    }
    await addMemeToLib(picPath.value[picIdx.value], editorSummary.value, editorDesc.value, editorTags.value, editorDeleteFile.value)
    nextPicture()
  }catch(e){
    ElMessage.error(String(e))
  }
}



</script>
<template lang="pug">
el-container.viewport(v-if="picIdx != -1 && picIdx != picPath.length")
  el-header
    m-titlebar(:back="true" :title="`Bulk add: ${basename}(${picIdx+1}/${picPath.length})`")
      template(#default)
        el-button-group
          el-button(@click="nextPicture") 
            font-awesome-icon(icon="fa-solid fa-trash")
            span(style="margin-left: 6px;") Skip
          el-button(@click="addMeme") 
            font-awesome-icon(icon="fa-solid fa-arrow-right")
            span(style="margin-left: 6px;") Next
  el-main.bulk-main
    el-progress.bulk-progress(
      :text-inside="true"
      :stroke-width="26"
      :percentage="Math.ceil(picIdx / picPath.length * 100)")
    el-card.bulk-image-card
      el-image.bulk-image(
        fit="contain"
        :src="imageLocalURL")
    .bulk-editor
      meme-basic-info-editor(
        v-model:delete-file="editorDeleteFile"
        v-model:summary="editorSummary"
        v-model:description="editorDesc"
        v-model:tags="editorTags")
.viewport(
  v-else-if="picIdx == picPath.length"
  style="display: flex; align-items: center; justify-content: center;")
  el-result(
    icon="success"
    title="You have added all images to library")
    template(#extra)
      el-button(
        type="primary"
        @click="$router.back()") Back


</template>

<style scoped lang="scss">
.viewport{
  width: 100%;
  height: 100%;
}
.bulk-main{
  display: grid;
  height: 100%;

  gap: 12px;
  grid-template-columns: repeat(2, 1fr);
  grid-template-rows: auto 1fr;

  .bulk-progress{
    grid-row: 1;
    grid-column: 1 / 3;
  }
  .bulk-image-card{
    grid-row: 2;
    grid-column: 1;

    :deep(.el-card__body) {
      width: 100%;
      height: 100%;
    }
    .bulk-image{
      width: 100%;
      height: 100%;
    }
  }
  .bulk-editor{
    grid-row: 2;
    grid-column: 2;
    display: flex;
    justify-content: center;
    align-items: center;

    > * {
      flex-grow: 1;
    }
  }
}

</style>