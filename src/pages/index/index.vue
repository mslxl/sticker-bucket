<script setup lang="ts">
import * as dialog from '@tauri-apps/plugin-dialog'
import { uniq } from 'lodash/fp'
import { LucideSearch } from 'lucide-vue-next'
import { useRouter } from 'vue-router'
import LibraryHistoryView from '@/components/LibraryHistoryView.vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { useAppPrefs, useAppPrefsMutate } from '@/hooks/use-app-prefs'
import { commands } from '@/lib/client'

const router = useRouter()
const mutatePrefs = useAppPrefsMutate()
const { data: prefs, isSuccess, isFetching, isError, error: errorMsg } = useAppPrefs()

async function handleOpenLibrary(path: string) {
  try {
    await commands.openDatabase(path)
    await mutatePrefs((prefs) => {
      prefs.database_history = uniq([path, ...prefs.database_history])
    })
    router.push('/dash')
  }
  catch (e) {
    await dialog.message(e as string, {
      title: 'Error',
      kind: 'error',
    })
  }
}

async function handleOpenFolder() {
  const userChosen = await dialog.open({
    directory: true,
    multiple: false,
  })
  if (userChosen == null)
    return
  await handleOpenLibrary(userChosen)
}
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="relative m-4 flex items-center gap-2">
      <LucideSearch class="absolute left-2 top-2.5 size-4 text-muted-foreground" />
      <Input class="h-9 flex-1 pl-8" />
      <Button @click="handleOpenFolder()">
        Open
      </Button>
    </div>
    <Separator />
    <LibraryHistoryView v-if="isSuccess" :history="prefs!.database_history" @open-library="handleOpenLibrary" />
    <ul v-else-if="isFetching" class="p-4 space-y-2">
      <li v-for="idx in 3" :key="idx">
        <Skeleton class="h-4 w-full" />
      </li>
    </ul>
    <div v-else-if="isError" class="relative">
      <div class="absolute right-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2">
        Error: {{ errorMsg }}
      </div>
    </div>
  </div>
</template>
