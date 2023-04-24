<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { computed, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'

import { library } from '@fortawesome/fontawesome-svg-core'
import { faSquarePlus, faTurnDown, faCheck } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import { openImageAndInterfer, addMemeToLib, queryNamespaceWithPrefix, queryTagValueWithPrefix } from '../scripts/rs/meme'

import MButton from '../components/basic/Button.vue'
import MCard from '../components/basic/Card.vue'
import MTitleBar from '../components/basic/TitleBar.vue'
import MInput from '../components/basic/InputField.vue'
import MTag from '../components/basic/Tag.vue'
import MCheckbox from '../components/basic/Checkbox.vue'
import MAutoComplate from '../components/basic/AutoComplate.vue'


library.add(faSquarePlus, faTurnDown, faCheck)
const imageRealPath = ref('')
const imageSummary = ref('')
const imageDesc = ref('')
const imageTags = reactive<any>({})
const imageNamespace = computed(() => Object.keys(imageTags))
const deleteFileAfterAdd = ref(true)

const imagePath = computed(() => imageRealPath.value.length > 0 ? convertFileSrc(imageRealPath.value) : '')
const router = useRouter()

const tagAutoComplate = ref<InstanceType<typeof MAutoComplate>>()


async function chooseImage() {
  let meme = await openImageAndInterfer()
  if (meme) {
    imageRealPath.value = meme.path
    if (meme.summary) imageSummary.value = meme.summary
    if (meme.desc) imageDesc.value = meme.desc
  }
}

const tagInput = ref('')

async function onTagInput(value: string) {
  let coloIdx = value.indexOf(':')
  if (coloIdx != -1) {
    let namespace = value.substring(0, coloIdx)
    let tagValue = await queryTagValueWithPrefix(namespace, value.substring(coloIdx + 1))
    tagAutoComplate.value?.update(tagValue.map(n => { return { value: n, tip: 'Value' } }))
  } else {
    let namespace = await queryNamespaceWithPrefix(value)
    tagAutoComplate.value?.update(namespace.map(n => { return { value: n, tip: 'Namespace' } }))
  }

}

function acceptAutoComplate(value: string) {
  let coloIdx = tagInput.value.indexOf(':')
  if (coloIdx != -1) {
    tagInput.value = tagInput.value.substring(0, coloIdx + 1) + value
    disposeAutoComplate()
  } else {
    tagInput.value = value + ":"
    disposeAutoComplate()
    onTagInput(tagInput.value)
  }
}

function disposeAutoComplate() {
  setTimeout(() => {
    if (!tagInput.value.trim().endsWith(':')) {
      tagAutoComplate.value?.dispose()
    }
  }, 250)
}

function addNewTag() {
  const legalRegex = /(.+):(.+)/
  const result = legalRegex.exec(tagInput.value)
  if (!result) return
  const namespace = result[1].trim()
  const value = result[2].trim()
  if (imageTags[namespace]) {
    imageTags[namespace].push(value)
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
    img.image(v-if="imagePath.trim && imagePath.trim().length != 0" :src="imagePath")
    font-awesome-icon.image.svg-image(v-else icon="fa-solid fa-square-plus")

  .editor
    div
    m-checkbox(label="Delete file after add" v-model="deleteFileAfterAdd")

    span Summary
    m-input(v-model="imageSummary")

    span Description
    m-input(v-model="imageDesc")

    span Tag
    m-auto-complate(ref="tagAutoComplate" @accpet="acceptAutoComplate")
      m-input(v-model="tagInput" @focusout="disposeAutoComplate()" @input="onTagInput($event)")
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
  width: 100%;
  overflow: auto;
  position: static;
  display: flex;

  flex-direction: column;
}


.image-viewer,
.editor {
  margin: 24px;
}

@media screen and (max-width: 420px) {
  .editor {
    grid-template-columns: 1fr !important;
  }
}

.editor {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 8px;
}

.image-viewer {
  position: static;
  height: 50%;

  display: flex;
  justify-content: center;
  align-items: center;
  padding: 12px;

  .image {
    display: block;
    height: 100%;
  }
  .svg-image{
    max-height: 240px;
    max-width: 240px;
  }
}
@media screen and (min-width: 1080px) {
  .panel {
    flex-direction: row;
    flex-wrap: nowrap;
  }
  .image-viewer{
    flex: 1;
    align-self: stretch;
    height: auto;
    width: auto;
    .image{
      height: auto;
      width: 100%;
    }
  }
  .editor{
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