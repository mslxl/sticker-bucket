<script setup lang="ts">
import { ref } from 'vue'
const props = withDefaults(defineProps<{
  label?: string,
  modelValue?: boolean
}>(), {
  label: "",
  modelValue: false
})

const emits = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

function toggle(){
  emits('update:modelValue', !props.modelValue)
}

const id = ref(Math.random().toString())

</script>
<template lang="pug">
span.checkbox(@click="toggle")
  input(type="checkbox" :checked="modelValue" :id="id" @input="toggle()")
  label(v-if="props.label.trim().length  > 0" :for="id") {{ props.label.trim() }}
</template>

<style scoped lang="scss">
.checkbox {
  display: flex;
  align-items: center;
  input {
    background: var(--primary-background-color);
    border: 1px solid var(--primary-border-color);
    outline: none;
    width: 1em;
    height: 1em;
  }

  label {
    line-height: 1em;
    margin-left: 1em;
  }
}
</style>