import {createRouter, createWebHashHistory} from 'vue-router'

const routes = [
  { path : '/', component: () => import('./views/Main.vue'), name: 'main' },
  { path : '/settings', component: () => import('./views/Settings.vue'), name: 'settings' },
  { path : '/meme/add', component: () => import('./views/MemeAdd.vue'), name: 'meme.add'}
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})
export default router