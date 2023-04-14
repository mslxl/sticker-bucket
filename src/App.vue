<script setup lang="ts">
import { RouterView, useRouter } from 'vue-router'
import { useSettings } from './store/settings'

import Sidebar from './components/Sidebar.vue'



const settings = useSettings()
const router = useRouter()

router.afterEach((to, from)=>{
  const toDepth = to.path.split('/').length
  const fromDepth = from.path.split('/').length
  to.meta.transition = toDepth < fromDepth ? 'slide-right' : 'slide-left'
})

</script>

<template lang="pug">
.container(:data-theme="settings.theme" :data-colorschema="settings.colorschema")
  sidebar.sidebar
  .main-view
    router-view(v-slot="{ Component, route }")
      transition(enterActiveClass="animate__animated animate__faster animate__fadeIn" leaveActiveClass="animate__animated animate__faster animate__fadeOut")
        component.viewport(:is="Component" :key="route.path")
</template>

<style scoped lang="scss">

.main-view {
  position: relative;
  height: 100vh;
  flex-grow: 1;
  background: var(--primary-background-color);
  overflow: hidden;
  border-radius: 16px;

  .viewport{
    position: absolute;
    width: 100%;
    height: 100%;
  }
}

.container {
  width: 100vw;
  height: 100vh;
  background-color: var(--secondary-background-color);
  display: flex;
  flex-direction: row;
}
</style>
