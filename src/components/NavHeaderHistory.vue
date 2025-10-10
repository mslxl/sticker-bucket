<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import * as dialog from '@tauri-apps/plugin-dialog'
import { capitalize, compose, includes, join, take, takeLastWhile, uniq } from 'lodash/fp'
import {
  ChevronsUpDown,
  Plus,
} from 'lucide-vue-next'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { SidebarMenuButton } from '@/components/ui/sidebar'
import { Skeleton } from '@/components/ui/skeleton'

import { useAppPrefs, useAppPrefsMutate } from '@/hooks/use-app-prefs'
import { commands } from '@/lib/client'

const makeItemName = compose([join(''), takeLastWhile<string>(x => !includes(x, ['/', '\\']))])
const makeAvatorFallBack = compose([capitalize, join(''), take(2), makeItemName])

const { status: prefsStatus, data: prefs } = useAppPrefs()
const { data: database, status: databaseStatus } = useQuery({
  queryKey: ['database'],
  queryFn: () => commands.getDatabase(),
})
const mutatePrefs = useAppPrefsMutate()

async function handleOpenLibrary(path: string) {
  try {
    await commands.openDatabase(path)
    await mutatePrefs((prefs) => {
      prefs.database_history = uniq([path, ...prefs.database_history])
    })
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
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <SidebarMenuButton
        size="lg"
        class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
      >
        <div class="bg-sidebar-primary text-sidebar-primary-foreground mr-4 flex aspect-square size-8 items-center justify-center rounded-lg">
          <Avatar v-if="databaseStatus === 'success'">
            <AvatarFallback>{{ makeAvatorFallBack(database!) }}</AvatarFallback>
          </Avatar>
          <Skeleton v-else class="h-8 w-8 rounded-full" />
        </div>
        <div class="grid flex-1 text-left text-sm leading-tight">
          <span v-if="databaseStatus === 'success'" class="truncate font-semibold">{{ makeItemName(database!) }}</span>
          <span v-if="databaseStatus === 'success'" class="truncate text-xs">{{ database! }}</span>
        </div>
        <ChevronsUpDown class="ml-auto" />
      </SidebarMenuButton>
    </DropdownMenuTrigger>
    <DropdownMenuContent
      class="w-[--radix-dropdown-menu-trigger-width] min-w-56 rounded-lg"
      align="start"
      side="bottom"
      :side-offset="4"
    >
      <DropdownMenuLabel class="text-xs text-muted-foreground">
        History Library
      </DropdownMenuLabel>
      <DropdownMenuItem
        v-for="item of prefs!.database_history"
        v-if="prefsStatus === 'success'"
        :key="item"
        class="gap-2 p-2"
        @click="handleOpenLibrary(item)"
      >
        <div class="mr-4 flex size-6 items-center justify-center rounded-sm border">
          <Avatar>
            <AvatarFallback>{{ makeAvatorFallBack(item) }}</AvatarFallback>
          </Avatar>
        </div>
        {{ item }}
      </DropdownMenuItem>
      <DropdownMenuSeparator />
      <DropdownMenuItem class="gap-2 p-2" @click="handleOpenFolder()">
        <div class="flex size-6 items-center justify-center rounded-md border bg-background">
          <Plus class="size-4" />
        </div>
        <div class="font-medium text-muted-foreground">
          Open
        </div>
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
