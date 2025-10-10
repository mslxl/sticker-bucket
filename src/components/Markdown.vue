<script setup lang="ts">
import markdownit from 'markdown-it'
import { cn } from '@/lib/utils'

const props = withDefaults(defineProps<{
  content: string
  env?: any
  inline?: boolean
  class?: string
}>(), {
  env: () => ({}),
  inline: false,
  class: '',
})

const md = markdownit()

const parsed = props.inline ? md.renderInline(props.content, props.env) : md.render(props.content, props.env)
</script>

<template>
  <div :class="cn('markdown-body', props.class)" v-html="parsed" />
</template>

<style scoped>
@import 'github-markdown-css/github-markdown.css';
</style>
