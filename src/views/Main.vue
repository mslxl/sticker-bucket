<script setup lang="ts">
import { library } from '@fortawesome/fontawesome-svg-core'
import { faAdd, faPen, faX, faEllipsisV } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'


import MDivider from '../components/basic/Divider.vue'
import MButton from '../components/basic/Button.vue'
import MTitleBar from '../components/basic/TitleBar.vue'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api'

library.add(faAdd, faPen, faX, faEllipsisV)

</script>
<template lang="pug">
m-title-bar(title="All")
  m-button.btn-item.btn-add(@click="$router.push({name: 'meme.add'})")
    font-awesome-icon(icon="fa-solid fa-add")
    span Add
  m-divider(:vertical="true" :dark="true")
  m-button.btn-item.btn-edit
    font-awesome-icon(icon="fa-solid fa-pen")
    span Edit
  m-button.btn-item.btn-remove
    font-awesome-icon(icon="fa-solid fa-x")
    span Remove
  m-button.btn-item.btn-more 
    font-awesome-icon(icon="fa-solid fa-ellipsis-vertical")
</template>


<style scoped lang="scss">
.btn-item {
  overflow: hidden;
  white-space: nowrap;
  line-height: 20px;

  span {
    padding: 0 8px 0 8px;
  }
}
</style>