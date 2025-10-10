<script setup lang="ts">
import type { VirtualizerOptions } from '@tanstack/vue-virtual'
import { elementScroll, useVirtualizer } from '@tanstack/vue-virtual'
import { chunk, flatten, last, pipe } from 'lodash/fp'
import { BoxIcon, LucideLoader2 } from 'lucide-vue-next'
import { AspectRatio } from 'reka-ui'
import { computed, ref, useTemplateRef, watchEffect } from 'vue'
import Asset from '@/components/media/Asset.vue'
import { useCollectionsInfiniteQuery } from '@/hooks/use-collections'
import { useGridCols } from '@/hooks/use-grid-cols'

const { data, hasNextPage, fetchNextPage, status, error, isFetchingNextPage } = useCollectionsInfiniteQuery()

const { cols } = useGridCols()

const allRows = computed(() =>
  data.value ? pipe(flatten, chunk(cols.value))(data.value.pages) : [])

const parentRef = useTemplateRef('listParentRef')
const scrollingRef = ref<number>()

function easeInOutQuint(t: number) {
  return t < 0.5 ? 16 * t * t * t * t * t : 1 + 16 * --t * t * t * t * t
}
const scrollToFn: VirtualizerOptions<any, any>['scrollToFn'] = (
  offset,
  canSmooth,
  instance,
) => {
  const duration = 1000
  const start = parentRef.value?.scrollTop || 0
  const startTime = (scrollingRef.value = Date.now())

  const run = () => {
    if (scrollingRef.value !== startTime)
      return
    const now = Date.now()
    const elapsed = now - startTime
    const progress = easeInOutQuint(Math.min(elapsed / duration, 1))
    const interpolated = start + (offset - start) * progress

    if (elapsed < duration) {
      elementScroll(interpolated, canSmooth, instance)
      requestAnimationFrame(run)
    }
    else {
      elementScroll(interpolated, canSmooth, instance)
    }
  }

  requestAnimationFrame(run)
}

const rowVirtualizerOptions = computed(() => ({
  count: hasNextPage.value ? allRows.value.length + 1 : allRows.value.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 100,
  overscan: 5,
  scrollToFn,
}))

const rowVirtualizer = useVirtualizer(
  rowVirtualizerOptions,
)

const virtualRows = computed(() => rowVirtualizer.value.getVirtualItems())
const totalSize = computed(() => rowVirtualizer.value.getTotalSize())

watchEffect(() => {
  const lastItem = last(virtualRows.value)
  if (!lastItem)
    return
  if (lastItem.index >= allRows.value.length - 1
    && hasNextPage.value
    && !isFetchingNextPage.value
  ) {
    fetchNextPage()
  }
})
function measureElement(el: HTMLDivElement) {
  if (!el) {
    return
  }

  rowVirtualizer.value.measureElement(el)

  return undefined
}
</script>

<template>
  <p v-if="status === 'pending'">
    Loading...
  </p>
  <p v-else-if="status === 'error'">
    Error: {{ error }}
  </p>
  <div v-else ref="listParentRef" class="size-full overflow-y-auto overflow-x-hidden">
    <div class="relative w-full" :style="{ height: `${totalSize}px` }">
      <div
        v-for="vrow in virtualRows" :key="vrow.key.toString()" class="absolute left-0 top-0 w-full" :style="{
          height: `${vrow.size}px`,
          transform: `translateY(${vrow.start}px)`,
        }"
      >
        <template v-if="vrow.index > allRows.length - 1">
          <div v-if="hasNextPage" class="text-center">
            <LucideLoader2 class="inline animate-spin" />
          </div>
          <div v-else>
            End of World
          </div>
        </template>
        <template v-else>
          <div
            :ref="measureElement"
            :data-index="vrow.index"
            class="grid w-full p-2"
            :style="{
              'display': 'grid',
              'gap': '12px',
              'grid-template-columns': `repeat(${cols}, minmax(0, 1fr))`,
            }"
          >
            <RouterLink v-for="item in allRows[vrow.index]" :key="item.id" :to="`/meme/${item.id}`">
              <div class="flex flex-col pb-1 border rounded-md overflow-clip">
                <AspectRatio :ratio="16 / 9" class="overflow-clip">
                  <Asset v-if="item.preview" :asset="item.preview" />
                  <BoxIcon v-else class="size-full" />
                </AspectRatio>
                <div class="space-y-1 p-2 text-left text-sm leading-tight bg-background border-t">
                  <span class="block truncate font-semibold">{{ item.name }}</span>
                  <span class="truncate text-xs text-muted-foreground"> @{{ item.author }}</span>
                </div>
              </div>
            </RouterLink>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>
