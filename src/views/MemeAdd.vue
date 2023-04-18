<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { computed, reactive, ref} from 'vue'
import { useRouter } from 'vue-router'

import { library } from '@fortawesome/fontawesome-svg-core'
import { faSquarePlus, faTurnDown, faCheck } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import { openImageAndInterfer, addMemeToLib } from '../scripts/rs/meme'

import MButton from '../components/basic/Button.vue'
import MCard from '../components/basic/Card.vue'
import MTitleBar from '../components/basic/TitleBar.vue'
import MInput from '../components/basic/InputField.vue'
import MTag from '../components/basic/Tag.vue'
import MCheckbox from '../components/basic/Checkbox.vue'


library.add(faSquarePlus, faTurnDown, faCheck)
const imageRealPath = ref('')
const imageSummary = ref('')
const imageDesc = ref('')
const imageTags = reactive<any>({})
const imageNamespace = computed(() => Object.keys(imageTags))
const deleteFileAfterAdd = ref(true)

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

const tagInput = ref('')
function addNewTag() {
  const legalRegex = /(.+):(.+)/
  const result = legalRegex.exec(tagInput.value)
  if (!result) return
  const namespace = result[1].trim()
  const value = result[2].trim()
  if (imageTags[namespace]) {
    imageTags[namespace].append(value)
  } else {
    imageTags[namespace] = [value]
  }
  tagInput.value = ''
}

function removeTag(namespace: string, index: number) {
  imageTags[namespace].splice(index, 1)
  if (imageTags[namespace].length == 0) {
    delete imageTags[namespace]
  }
}

const addBtnAvailable = computed(() => imagePath.value.length > 0 && imageSummary.value.trim().length > 0)

async function addMeme() {
  if (!addBtnAvailable.value) {
    return
  }

  let tagList = []
  for (let key of Object.keys(imageTags)) {
    let value = imageTags[key] as string[]
    for (let item of value) {
      tagList.push({
        namespace: key,
        value: item
      })
    }
  }

  addMemeToLib(imageRealPath.value, imageSummary.value, imageDesc.value, tagList, deleteFileAfterAdd.value)

  router.back()
}

</script>

<template lang="pug">
m-title-bar(title="Add" :back="true")
  m-button.btn-item(@click="addMeme")
    font-awesome-icon(icon="fa-solid fa-check")
    span Save

.panel
  m-card.image-viewer(@click="chooseImage")
    .image__overlay()
      img.image(v-if="imagePath.trim && imagePath.trim().length != 0" :src="imagePath")
      font-awesome-icon.image(v-else icon="fa-solid fa-square-plus")

  .editor
    div
    m-checkbox(label="Delete file after add" v-model="deleteFileAfterAdd")

    span Summary
    m-input(v-model="imageSummary")

    span Description
    m-input(v-model="imageDesc")

    span Tag
    m-input(v-model="tagInput")
      m-button(@click="addNewTag()")
        font-awesome-icon(icon="fa-solid fa-turn-down fa-rotate-90")
    //- tag show   
    template(v-for="nsp in imageNamespace" :key="nsp")
      m-tag {{  nsp }}
      .tag-value
        m-tag.tag-item(v-for="(value, valueIndex) in imageTags[nsp]" :key="nsp+value" :closable="true" @on-close="removeTag(nsp, valueIndex)") {{  value  }}

</template>


<style scoped lang="scss">
.tag-value {
  .tag-item {
    margin-left: 2px;
  }
}

.panel {
  height: 100%;
  overflow: auto;
  position: relative;
}

.image-viewer,
.editor {
  margin: 24px;
  position: relative;
}

@media only screen and (max-width: 420px) {
  .editor {
    grid-template-columns: 1fr !important;
  }

}

.editor {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 8px;
}

.image__overlay {
  position: absolute;
  display: flex;
  justify-content: center;
  height: 100%;
  width: 100%;
  padding: 12px;
  text-align: center;
}

.image-viewer {
  min-height: 128px;
  min-width: 128px;
  max-height: 30%;

  .image {
    display: block;
    height: 100%;
  }
}

.btn-item {
  line-height: 20px;

  span {
    padding: 0 8px 0 8px;
  }
}
</style>