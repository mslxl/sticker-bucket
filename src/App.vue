<script setup lang="ts">
import {RouterView} from 'vue-router'
</script>

<template lang="pug">
.container
  router-view
</template>

<style scoped lang="scss">
.container {
  width: 100vw;
  height: 100vh;
}
</style>
