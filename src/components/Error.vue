<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'

const props = withDefaults(defineProps<{
  error?: Error | string
}>(), {
  error: 'Internal Error',
})

const canvas = ref<HTMLCanvasElement>()

const message = computed(() => {
  if (props.error instanceof Error) {
    return props.error.message
  }
  return props.error as string
})

const bg = ref('')
const bgUrl = computed(() => `url(${bg.value})`)
const height = ref('100%')
const width = ref('100%')

onMounted(() => {
  const ctx = canvas.value!.getContext('2d')!
  ctx.fillStyle = 'yellow'
  ctx.lineWidth = 4
  ctx.font = '1em Arial'
  const metrics = ctx.measureText(message.value)

  width.value = `${metrics.width + 12}px`
  height.value = `${metrics.actualBoundingBoxAscent + 12}px`

  nextTick(() => {
    ctx.fillStyle = 'yellow'
    ctx.font = '1em Arial'

    ctx.fillText(message.value, 0, metrics.actualBoundingBoxAscent)
    bg.value = canvas.value!.toDataURL()
  })
})
</script>

<template>
  <div class="error-container">
    <canvas ref="canvas" :width="width" :height="height" class="error-canvas" />
    <div class="error-message" />
  </div>
</template>

<style scoped>
.error-container{
    width: 100%;
    height: 100%;

    background-color: red;
    border: 4px dashed yellow;
    padding: 6px;
}
.error-canvas{
    display: none;
}
.error-message{
    width: 100%;
    height: 100%;
    background-image: v-bind('bgUrl');
}
</style>
