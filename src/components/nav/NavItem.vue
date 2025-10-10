<script setup lang="ts">
import { cn } from '@/lib/utils'

interface Props {
  to: string
}

const props = defineProps<Props>()
</script>

<template>
  <li>
    <RouterLink
      :class="cn(
        'inline-flex items-center whitespace-nowrap rounded-md text-sm font-medium transition-colors',
        'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring',
        'disabled:pointer-events-none disabled:opacity-50',
        'h-9 px-4 py-2 w-full justify-start',
        'hover:bg-accent hover:text-accent-foreground',
      )" active-class="bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80" :to="props.to"
    >
      <slot />
    </RouterLink>
  </li>
</template>
