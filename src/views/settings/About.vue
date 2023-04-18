
<script setup lang="ts">
// import { library } from '@fortawesome/fontawesome-svg-core'
// import { faAdd, faPen, faX, faEllipsisV } from '@fortawesome/free-solid-svg-icons'
// import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { ref } from 'vue'

import MTitleBar from '../../components/basic/TitleBar.vue'
import MCard from '../../components/basic/Card.vue'

import { getDataDir, getSQLiteVersion, getTableVersion } from '../../scripts/rs/db'


const itemTableVersionCode = ref(-1)
const itemDatabseDir = ref('')
const itemSQLiteVersion = ref('')

function showOrTip(v: any): any {
  if (typeof v == 'number' && v == -1)
    return "initialize..."

  if (typeof v == 'string' && v.trim().length == 0)
    return "initialize..."
  return v
}

getTableVersion().then(code => itemTableVersionCode.value = code)
getDataDir().then(dir => itemDatabseDir.value = dir)
getSQLiteVersion().then(version => itemSQLiteVersion.value = version)

</script>
<template lang="pug">
m-title-bar(title="About" :back="true")

.panel
  m-card.preference-item
    span Data Location
    .space
    span.selectable {{ showOrTip(itemDatabseDir) }}
  m-card.preference-item
    span Bundled Database
    .space
    span.selectable {{ showOrTip(itemSQLiteVersion) }}
  m-card.preference-item
    span Database Table Version
    .space
    span.selectable {{ showOrTip(itemTableVersionCode) }}

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