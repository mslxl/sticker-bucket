<script setup lang="ts">
import { library } from '@fortawesome/fontawesome-svg-core'
import { faArrowLeft } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import MDivider from './Divider.vue'
import MButton from './Button.vue'

library.add(faArrowLeft)

const props = withDefaults(defineProps<{
  title: string,
  back?: boolean
}>(), {
  back: false
})

</script>
<template lang="pug">
.button-bar-wrapper
  .button-bar
    m-button.btn-back(v-if="props.back && $router.back" @click="$router.back && $router.back()")
      font-awesome-icon(icon="fa-solid fa-arrow-left")
    span {{ props.title }}
    .space
    slot
  m-divider(:dark="true")
</template>

<style scoped lang="scss">
.button-bar {
  margin: 6px 12px 6px 12px;
  height: 36px;
  line-height: 36px;
  display: flex;
  .btn-back{
    margin-right: 12px;
    line-height: normal;
  }

  .space {
    flex-grow: 1;
  }

}
</style>