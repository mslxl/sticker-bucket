<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/tauri'

import { library } from '@fortawesome/fontawesome-svg-core'
import { faSquarePlus, faTurnDown, faCheck } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import { open_image_and_interfer } from '../scripts/rs/meme'

import MButton from '../components/basic/Button.vue'
import MCard from '../components/basic/Card.vue'
import MTitleBar from '../components/basic/TitleBar.vue'
import MInput from '../components/basic/InputField.vue'
import MTag from '../components/basic/Tag.vue'


library.add(faSquarePlus, faTurnDown, faCheck)
const imagePath = ref('')
const imageSummary = ref('')
const imageDesc = ref('')
const imageTags = reactive<any>({})
const imageNamespace = computed(() => Object.keys(imageTags))

imageTags['female'] = ['lolicon', 'keminomimi', 'cat girl']
imageTags['character'] = ['yukikaze']

console.log(imageTags)

async function chooseImage() {
  let meme = await open_image_and_interfer()
  if (meme) {
    // imageBase64.value = meme.base64
    imagePath.value = convertFileSrc(meme.path)
    if (meme.summary) imageSummary.value = meme.summary
    if (meme.desc) imageDesc.value = meme.desc
  }
}

</script>

<template lang="pug">
m-title-bar(title="Add" :back="true")
  m-button.btn-item
    font-awesome-icon(icon="fa-solid fa-check")
    span Save

.panel
  m-card.image-viewer(@click="chooseImage")
    .image__overlay()
      img.image(v-if="imagePath.trim && imagePath.trim().length != 0" :src="imagePath")
      font-awesome-icon.image(v-else icon="fa-solid fa-square-plus")

  .editor
    span Name
    m-input

    span Summary
    m-input

    span Tag
    m-input
      m-button
        font-awesome-icon(icon="fa-solid fa-turn-down fa-rotate-90")
    //- tag show   
    template(v-for="nsp in imageNamespace")
      m-tag {{  nsp }}
      .tag-value
        m-tag.tag-item(v-for="value in imageTags[nsp]" :key="nsp+value" :closable="true") {{  value  }}

</template>


<style scoped lang="scss">

.tag-value{
  .tag-item{
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