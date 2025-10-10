<script setup lang="ts">
import { LucideLibrary } from 'lucide-vue-next'
import AppIcon from '@/components/AppIcon.vue'
import { Nav, NavItem } from '@/components/nav'

const navItems = [{
  icon: LucideLibrary,
  title: 'Library',
  to: '/',
}]
</script>

<template>
  <div class="flex h-screen w-screen select-none items-stretch">
    <div class="flex h-full flex-col">
      <Suspense>
        <AppIcon />
      </Suspense>
      <aside class="flex-1 border-r p-4 pb-12">
        <Nav>
          <NavItem v-for="item in navItems" :key="item.to" :to="item.to">
            <component :is="item.icon" class="mr-2 size-4" />
            {{ item.title }}
          </NavItem>
        </Nav>
      </aside>
    </div>
    <div class="mt-[32px] flex-1">
      <RouterView />
    </div>
  </div>
</template>
