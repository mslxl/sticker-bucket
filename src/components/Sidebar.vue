<script setup lang="ts">
import MButton from './basic/Button.vue'
import MDivider from './basic/Divider.vue'

import { library } from '@fortawesome/fontawesome-svg-core'
import { faBars, faInbox, faTrash, faHeart, faGear } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import { ref } from 'vue'

library.add(faBars, faInbox, faTrash, faHeart, faGear)
const isExpand = ref(false)



</script>
<template lang="pug">
.sidebar(:class="{expand: isExpand}")
  ul.sidebar-list
    li
      m-button.menu-btn(@click="isExpand = !isExpand" justify="left") 
        font-awesome-icon.icon(icon="fa-solid fa-bars")
    li
      m-button.menu-btn(justify="left") 
        font-awesome-icon.icon(icon="fa-solid fa-inbox")
        span(v-show="isExpand") All
    li
      m-button.menu-btn(justify="left") 
        font-awesome-icon.icon(icon="fa-solid fa-heart")
        span(v-show="isExpand") Fav
    li 
      m-divider
    li
      m-button.menu-btn(justify="left") 
        font-awesome-icon.icon(icon="fa-solid fa-trash")
        span(v-show="isExpand") Trash
  .space
  ul.sidebar-list
    li
      m-button.menu-btn(justify="left") 
        font-awesome-icon.icon(icon="fa-solid fa-gear")
        span(v-show="isExpand") Settings 
</template>

<style scoped lang="scss">
.sidebar {
  width: 64px;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--secondary-background-color);
  padding-top: 4px;
  padding-bottom: 4px;

  transition: width .3s;

  z-index: 999999;

  * {
    width: 100%;
  }

  .space{
    flex-grow: 1;
  }
}

.menu-btn {
  .icon {
    width: 42px !important;
  }

  white-space: nowrap;
}

.sidebar-list {
  display: block;
  list-style: none;
  li {
    text-decoration: none;
    padding: 2px 4px 2px 2px;
  }
}

.expand {
  width: 30%;
}
</style>