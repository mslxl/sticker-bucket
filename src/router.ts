import {createRouter, createWebHashHistory} from 'vue-router'

const routes = [
  { path : '/', component: () => import('./views/Main.vue'), name: 'main' },
  { path : '/meme/add', component: () => import('./views/MemeAdd.vue'), name: 'meme.add'},

  // View
  { path : '/meme/view/{:id}', component: () => import('./views/MemeView.vue'), name: 'meme.view'},
  // Settings
  { path : '/settings', component: () => import('./views/settings/Main.vue'), name: 'settings' },
  { path : '/settings/about', component: () => import('./views/settings/About.vue'), name: 'settings.about' },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})
export default router