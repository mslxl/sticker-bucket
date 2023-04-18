<script setup lang="ts">
import MTitleBar from '../../components/basic/TitleBar.vue'
import MCard from '../../components/basic/Card.vue'

</script>
<template lang="pug">
m-title-bar(title="Settings")

.panel
  m-card.preference-item(@click="$router.push({name: 'settings.about'})")
    span About
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
</style>