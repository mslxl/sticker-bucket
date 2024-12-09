<script setup lang="ts">
import { useRoute } from 'vue-router'
import Error from '@/components/Error.vue'

const route = useRoute()

const search = route.params.search as string ?? ''
</script>

<template>
  <Error :error="`Unimplemented: ${search}`" />
</template>
