<script setup lang="ts">
import { app } from '@tauri-apps/api'
import logo from '@/../src-tauri/icons/128x128.png'

const appName = await app.getName()
const appVersion = await app.getVersion()
const tauriVersion = await app.getTauriVersion()
</script>

<template>
  <div class="my-4 flex items-center space-x-4 p-4">
    <span class="relative flex size-10 shrink-0 overflow-hidden">
      <img
        class="transition-ease-in-out hover:transform-rotate-360 size-full border-primary transition-transform"
        :src="logo"
      >

    </span>
    <div>
      <h3 class="font-semibold leading-none tracking-tight">
        {{ appName }}
      </h3>
      <p class="text-sm text-muted-foreground">
        v{{ appVersion }} (Tauri v{{ tauriVersion }})
      </p>
    </div>
  </div>
</template>
