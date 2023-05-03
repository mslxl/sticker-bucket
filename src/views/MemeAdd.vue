<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { library } from '@fortawesome/fontawesome-svg-core'
import { faSquarePlus, faCheck } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import { addMemeToLib } from '../scripts/rs/db'
import { openImageAndInterfer } from '../scripts/rs/local'


import { ElCard, ElImage, ElButton } from 'element-plus'
import MButton from '../components/basic/Button.vue'
import MTitleBar from '../components/basic/TitleBar.vue'

import MemeInfoEditor from '../components/MemeBasicInfoEditor.vue'

library.add(faSquarePlus, faCheck)
const imageRealPath = ref('')


const deleteFileAfterAdd = ref(true)
const imageSummary = ref('')
const imageDesc = ref('')
const imageTags = ref<{ namespace: string, value: string }[]>([])
const imagePath = computed(() => imageRealPath.value.length > 0 ? convertFileSrc(imageRealPath.value) : '')
const router = useRouter()

async function chooseImage() {
  let meme = await openImageAndInterfer()
  if (meme) {
    imageRealPath.value = meme.path
    if (meme.summary) imageSummary.value = meme.summary
    if (meme.desc) imageDesc.value = meme.desc
  }
}

const addBtnAvailable = computed(() => imagePath.value.length > 0 && imageSummary.value.trim().length > 0)

async function addMeme() {
  if (!addBtnAvailable.value) {
    return
  }

  addMemeToLib(imageRealPath.value, imageSummary.value, imageDesc.value, imageTags.value, deleteFileAfterAdd.value)

  router.back()
}

</script>

<template lang="pug">
.viewport
  m-title-bar(title="Add" :back="true")
    el-button.btn-item(
      type=""
      text
      @click="addMeme")
      font-awesome-icon(icon="fa-solid fa-check")
      span Save
  .panel
    el-card.image-viewer(
      :body-style="{ padding: '0px'}"
      @click="chooseImage")
      el-image.image(
        fit="scale-down"
        :src="imagePath")
        template(#error)
          div(style="display: flex; align-items: center; justify-content: center; width: 100%; height: 100%;")
            font-awesome-icon.svg-image(icon="fa-solid fa-square-plus")
    meme-info-editor.editor(
      v-model:delete-file="deleteFileAfterAdd"
      v-model:summary="imageSummary"
      v-model:description="imageDesc"
      v-model:tags="imageTags")


</template>


<style scoped lang="scss">
.viewport {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;

  .panel {
    flex-grow: 1;
  }
}

.tag-value {
  .tag-item {
    margin-left: 2px;
  }
}


.panel {
  height: 100%;
  width: 100%;
  overflow: auto;
  position: static;
  display: flex;

  flex-direction: column;
  padding: 12px;
}

.image-viewer,
.editor {
  margin: 12px;
}

.image-viewer {
  overflow: visible;
}


@media screen and (max-width: 420px) {
  .editor {
    grid-template-columns: 1fr !important;
  }
}


.image-viewer {
  position: static;
  height: 50%;

  display: flex;
  justify-content: center;
  align-items: center;
  padding: 12px;

  :deep(.el-card__body) {
    width: 100%;
    height: 100%;
  }

  .image {
    display: block;
    height: 100%;
    widows: 100%;
  }

  .svg-image {
    height: 24px;
    width: 24px;
  }
}

@media screen and (min-width: 1080px) {
  .panel {
    flex-direction: row;
    flex-wrap: nowrap;
  }

  .image-viewer {
    flex: 1;
    align-self: stretch;
    height: auto;
    width: auto;
  }

  .editor {
    flex: 1;
    align-self: center;
  }
}

.btn-item {
  line-height: 20px;

  span {
    padding: 0 8px 0 8px;
  }
}
</style>