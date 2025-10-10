<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { convertFileSrc } from '@tauri-apps/api/core'
import { match } from 'ts-pattern'
import Error from '@/components/Error.vue'
import Markdown from '@/components/Markdown.vue'
import { Skeleton } from '@/components/ui/skeleton'
import { type Asset, commands } from '@/lib/client'
import Image from './Image.vue'
import Video from './Video.vue'

const props = withDefaults(defineProps<{
  asset: Asset
  class?: string
}>(), {
  class: '',
})

const { isError, isLoading, isSuccess, data, error } = useQuery({
  queryKey: ['asset', props.asset.id],
  queryFn: async () =>
    match(props.asset).with({ type: 'text' }, async (value) => {
      return value.text
    }).with({ type: 'image' }, async (value) => {
      const image = convertFileSrc(await commands.resolvePath(value.image))
      return image
    }).with({ type: 'video' }, async (value) => {
      const video = convertFileSrc(await commands.resolvePath(value.video))
      return video
    }).exhaustive(),
})
</script>

<template>
  <Skeleton v-if="isLoading" :class="props.class" />
  <Image v-else-if="isSuccess && data && props.asset.type === 'image'" :src="data" :class="props.class" />
  <Video v-else-if="isSuccess && data && props.asset.type === 'video'" :src="data" :class="props.class" />
  <Markdown v-else-if="isSuccess && data && props.asset.type === 'text'" :content="data" />
  <Error v-else-if="isError" :error="error?.message ?? 'Unknown error'" />
</template>
