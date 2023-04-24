<script setup lang="ts">

const props = withDefaults(defineProps<{
  type?: 'text' | 'password',
  modelValue?: string
}>(), {
  type: 'text',
  modelValue: ''
})

const emits = defineEmits<{
  (e: 'update:modelValue', value: string):void
  (e: 'focusout'): void
  (e: 'input', value: string):void
}>()

function updateModelValue(event: EventTarget | null){
  emits('update:modelValue', (event as HTMLInputElement).value)
  emits('input', (event as HTMLInputElement).value)
}

</script>

<template lang="pug">
span.inputbox-wrapper
  input(:type="props.type" :value="modelValue" @input="updateModelValue($event.target)" @focusout="emits('focusout')")
  .suffix
    slot
</template>

<style scoped lang="scss">

.inputbox-wrapper{
  display: flex;
  justify-content: center;

  input{
    flex-grow: 1;
  }

  .suffix, input{
    margin: auto;
  }
  .suffix{
    padding: 2px;
  }



}

input {
  padding: 6px;

  width: 100%;
  height: 100%;

  background: var(--textfield-background);
  color: var(--font-color);
  border: 1.5px solid var(--textfield-background);
  border-bottom: 2px solid var(--textfield-background);
  border-radius: 6px;
  margin: 2px;
  outline: none;
  transition: .2s;
}

input:focus {
  background: var(--textfield-focus-background);
  border-bottom: 2px solid var(--primary-accent-color);
}
</style>