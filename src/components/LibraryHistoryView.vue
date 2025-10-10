<script setup lang="ts">
import { openPath } from '@tauri-apps/plugin-opener'
import { capitalize, compose, includes, join, take, takeLastWhile } from 'lodash/fp'
import { LucideMoreVertical } from 'lucide-vue-next'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'

const props = withDefaults(defineProps<{
  history?: string[]
}>(), {
  history: () => [],
})
const emit = defineEmits<{
  (e: 'openLibrary', path: string): void
  (e: 'remove', path: string): void
}>()
const makeItemName = compose([join(''), takeLastWhile<string>(x => !includes(x, ['/', '\\']))])
const makeAvatorFallBack = compose([capitalize, join(''), take(2), makeItemName])

function handleOpenLibrary(path: string) {
  emit('openLibrary', path)
}
function handleRevalInFiles(path: string) {
  openPath(path)
}
function handleRemove(path: string) {
  emit('remove', path)
}
</script>

<template>
  <ul class="overflow-y flex-1 px-8 py-2">
    <li
      v-for="item of props.history" :key="item"
      class="transition-background-color group my-4 flex items-center space-x-4 rounded-md p-2 hover:bg-accent hover:text-accent-foreground"
      @click="handleOpenLibrary(item)"
    >
      <Avatar>
        <AvatarFallback>
          {{ makeAvatorFallBack(item) }}
        </AvatarFallback>
      </Avatar>
      <div class="flex-1">
        <p class="text-sm tracking-tight">
          {{ makeItemName(item) }}
        </p>
        <p class="text-xs text-muted-foreground">
          {{ item }}
        </p>
      </div>
      <DropdownMenu>
        <DropdownMenuTrigger as-child>
          <Button class="opacity-0 transition-opacity group-hover:opacity-100" variant="outline" @click="$event.stopPropagation()">
            <LucideMoreVertical />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent>
          <DropdownMenuItem @click="handleOpenLibrary(item); $event.stopPropagation()">
            Open
          </DropdownMenuItem>
          <DropdownMenuItem @click="handleRevalInFiles(item); $event.stopPropagation()">
            Reval in Files
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem @click="handleRemove(item); $event.stopPropagation()">
            Remove
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </li>
  </ul>
</template>
