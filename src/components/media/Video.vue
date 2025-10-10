<script setup lang="ts">
import { useAttrs } from 'vue'

const props = defineProps<{
  src: string
}>()
const attrs = useAttrs()
</script>

<template>
  <div class="size-full relative">
    <!-- 背景模糊层 -->
    <video
      class="absolute inset-0 w-full h-full object-cover blur-lg scale-110 z-0"
      :src="props.src"
      alt="Video background"
      muted
      loop
      autoplay
      aria-hidden="true"
    />
    <!-- 前景视频层 -->
    <div class="h-full flex items-center justify-center relative z-10">
      <video
        class="h-full object-contain"
        :src="props.src" alt="Video" muted loop autoplay v-bind="attrs"
      />
    </div>
  </div>
</template>
