<script setup lang="ts">
import { reactive, ref } from 'vue';


const data = reactive<{ value: string, tip?: string }[]>([])
const display = ref(false)

function update(item: { value: string, tip?: string }[]) {
  display.value = true
  data.splice(0, data.length)
  data.push(...item)
}

function dispose() {
  data.splice(0, data.length)
  display.value = false
}

const emits = defineEmits<{
  (e: 'accpet', value: string): void
}>()

defineExpose({
  update,
  dispose
})
</script>

<template lang="pug">
.autocomplate
  slot.complate-input
  .complate-wrapper(:class="{deactive: !display}")
    .list
      .item(v-for="i in data" @click="emits('accpet', i.value)")
        span.content {{ i.value }}
        .space
        span.tip(v-if="i.tip") {{ i.tip }}
</template>

<style scoped lang="scss">
.deactive{
  display: none;
}

.complate-input,
.complate-wrapper,
.list {
  width: 100%;
  transition: all 0.3s;
}

.complate-wrapper{
  position: relative;
}

.list {
  position: absolute;
  z-index: 999;
  border: 1px solid var(--primary-border-color);
  .item:hover{
    background-color: var(--hover-color);
  }
  .item{
    padding: 18px 12px 18px 12px;
    height: 24px;
    line-height: 24px;
    background-color: var(--primary-background-color);
    display: flex;
    align-items: center;
    .content{
      font-size: large;
    }

    .space{
      flex-grow: 1;
    }
    .tip{
      font-size: smaller;
    }
  }
}
</style>