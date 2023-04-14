<script setup lang="ts">
import { library } from '@fortawesome/fontawesome-svg-core'
import { faX } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

library.add(faX)

const props = withDefaults(
  defineProps<{ closable?: boolean }>(), {
  closable: false
})

const emits = defineEmits<{
  (e: 'onClose'): void
}>()
</script>

<template lang="pug">
span.tag(:class="{closeable:props.closable}")
  span.content
    slot
  i.tag-icon(v-if="props.closable" @click="$emit('onClose')")
    font-awesome-icon(icon="fa-x")



</template>

<style scoped lang="scss">
.tag {
  display: inline-flex;
  height: 24px;
  line-height: 1;
  border-radius: 5px;
  padding: 0 9px;
  justify-content: center;
  align-items: center;
  white-space: nowrap;

  border: 1px solid var(--primary-border-color);

  .tag-icon {
    color: var(--font-color);
    height: 14px;
    width: 14px;
    display: inline-flex;

    justify-content: center;
    align-items: center;

    position: relative;
    fill: currentColor;
    border-radius: 50%;
    margin-left: 2px;
    svg{
      height: 7px !important;
      width: 7px !important;
      line-height: 1em;
      fill: currentColor;
    }
  }
  .tag-icon:hover{
    background-color: rgba($color: #fff, $alpha: .4);
  }
}

.tag.closable {
  padding-right: 3px;
}
</style>