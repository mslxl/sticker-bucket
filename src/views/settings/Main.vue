<script setup lang="ts">
import MTitleBar from '../../components/basic/TitleBar.vue'
import { ElRow, ElCol } from 'element-plus'
import { ElStatistic } from 'element-plus'
import { ElCard } from 'element-plus'

import { ref } from 'vue'
import { queryCountMemes, queryCountTags } from '../../scripts/rs/db'

const cntMemes = ref(-1)
const cntTags = ref(-1)

queryCountMemes().then(cnt => cntMemes.value = cnt)
queryCountTags().then(cnt => cntTags.value = cnt)



</script>
<template lang="pug">
m-title-bar(title="Settings")
.panel
  el-row.statistic
    el-col(:span="12")
      el-statistic(title="Total Memes" :value="cntMemes")
    el-col(:span="12")
      el-statistic(title="Total Tags" :value="cntTags")
    //- el-col(:span="8")
    //-   el-statistic(title="Total meme template" :value="0")
  el-card(
    @click="$router.push({name: 'settings.about'})")
    span About
</template>

<style scoped lang="scss">
.panel {
  padding: 24px;
}

.statistic {
  margin: 12px;
}
</style>