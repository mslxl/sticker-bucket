<script setup lang="ts">
import { watch, ref } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { getImageRealPath } from '../scripts/rs/meme'
import MCard from './basic/Card.vue'
import MDivider from './basic/Divider.vue'

const props = defineProps<{
  summary: string,
  imageId: string
}>()

const imageURL = ref('')

getImageRealPath(props.imageId).then((path) => {
  imageURL.value = convertFileSrc(path)
})

</script>

<template lang="pug">
m-card
  .content
    img.selectable(:src="imageURL")
    .title.selectable {{ props.summary }}
  m-divider
  slot

</template>

<style scoped lang="scss">
img{
  width: 100%;
}
.title{
  padding: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>