<script setup lang="ts">
import * as log from '@tauri-apps/plugin-log'
import { debounce } from 'lodash/fp'
import { LucideSearch } from 'lucide-vue-next'
import { ref } from 'vue'
import { onBeforeRouteUpdate, useRouter } from 'vue-router'
import { Input } from '@/components/ui/input'
import { Separator } from '@/components/ui/separator'
import { SidebarTrigger } from '@/components/ui/sidebar'

const router = useRouter()
const search = ref('')

onBeforeRouteUpdate((to, _, next) => {
  if (to.params.search) {
    search.value = to.params.search as string
  }
  else {
    search.value = ''
  }
  next()
})

function handleSearch() {
  log.info(`Searching for ${search.value.trim()}`)
  router.replace({
    path: `/dash/search/${search.value.trim()}`,
    force: true,
    replace: true,
  })
}

const debouncedSearch = debounce(500, handleSearch)
</script>

<template>
  <header class="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-[[data-collapsible=icon]]/sidebar-wrapper:h-12">
    <div class="flex items-center gap-2 px-4">
      <SidebarTrigger class="-ml-1" />
      <Separator orientation="vertical" class="mr-2 h-4" />

      <div class="relative flex items-center gap-2">
        <LucideSearch class="absolute left-2 top-2.5 size-4 text-muted-foreground" />
        <Input v-model="search" class="h-9 flex-1 pl-8" placeholder="Search..." @input="debouncedSearch" />
      </div>
    </div>
  </header>
</template>
