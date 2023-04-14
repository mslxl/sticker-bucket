
<script setup lang="ts">
// import { library } from '@fortawesome/fontawesome-svg-core'
// import { faAdd, faPen, faX, faEllipsisV } from '@fortawesome/free-solid-svg-icons'
// import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { ref } from 'vue'

import MTitleBar from '../components/basic/TitleBar.vue'
import MCard from '../components/basic/Card.vue'

import { getTableVersion } from '../scripts/rs/db'


const tableVersionCode = ref(-1)

getTableVersion().then(code => tableVersionCode.value = code)

</script>
<template lang="pug">
m-title-bar(title="Settings")

.panel
  m-card.preference-item
    span Database table version
    .space
    span {{ tableVersionCode == -1 ? "initialize database" : tableVersionCode }}

</template>

<style scoped lang="scss">
.panel {
  padding: 12px 24px 12px 24px;

  .preference-item {
    display: flex;
    padding: 12px;

    .space {
      flex: 1;
    }
  }

}

.btn-item {
  overflow: hidden;
  white-space: nowrap;
  line-height: 20px;

  span {
    padding: 0 8px 0 8px;
  }
}
</style>