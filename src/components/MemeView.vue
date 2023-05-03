<script setup lang="ts">
import { ref } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { getImageRealPath } from '../scripts/rs/db'
import { ElCard, ElImage, ElSkeleton } from 'element-plus'
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
el-card(:body-style="{ padding: '0px' }")
  el-image.img(
    :src="imageURL"
    loading="lazy"
    fit="fill")
    template(#placeholder)
      el-skeleton(
        :rows="5"
        animated)
  .content(style="padding: 14px;")
    .title.selectable {{ props.summary }}
  m-divider
  slot

</template>

<style scoped lang="scss">
.img {
  width: 100%;
  min-height: 6em;
  display: block;
}

.content{
  width: 100%;
  .title{
    text-align: center;
    width: 100%;
  }
}
.el-card{
  overflow: visible !important;
}
</style>