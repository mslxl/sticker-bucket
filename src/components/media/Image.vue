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
    <img
      class="absolute inset-0 w-full h-full object-cover blur-lg scale-110 z-0"
      alt="Image background"
      :src="props.src"
      aria-hidden="true"
    >
    <!-- 前景图片层 -->
    <div class="h-full flex items-center justify-center relative z-10">
      <img
        class="h-full object-contain"
        alt="Image"
        :src="props.src"
        v-bind="attrs"
      >
    </div>
  </div>
</template>
