<script setup lang="ts">
import { library } from '@fortawesome/fontawesome-svg-core'
import { faArrowLeft } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import { ElButton } from 'element-plus'
import MDivider from './Divider.vue'

library.add(faArrowLeft)

const props = withDefaults(defineProps<{
  title: string,
  back?: boolean,
  showButtonBar?:boolean
}>(), {
  back: false,
  showButtonBar: true
})

</script>
<template lang="pug">
.button-bar-wrapper
  .button-bar
    el-button.btn-back(
      v-if="props.back && $router.back"
      type=""
      text
      @click="$router.back && $router.back()")
      font-awesome-icon(icon="fa-solid fa-arrow-left")
    span {{ props.title }}
    .space
      slot(name="content")
    slot
  m-divider(:dark="true")
</template>

<style scoped lang="scss">
.button-bar {
  margin: 6px 12px 6px 12px;
  display: flex;
  align-items: center;
  min-height: 32px;
  .btn-back{
    margin-right: 12px;
    line-height: normal;
  }

  .space {
    flex-grow: 1;
  }

}
</style>