import {createRouter, createWebHashHistory} from 'vue-router'

const routes = [
  { path : '/', component: () => import('./views/Main.vue'), name: 'main' },
  { path : '/fav', component: () => import('./views/Main.vue'), name: 'fav' },
  { path : '/meme/add', component: () => import('./views/MemeAdd.vue'), name: 'meme.add'},
  { path : '/meme/add/bulk', component: () => import('./views/MemeBulkAdd.vue'), name: 'meme.add.bulk' },
  // View
  { path : '/meme/view/{:id}', component: () => import('./views/MemeView.vue'), name: 'meme.view'},
  // Edit
  { path: '/meme/edit/info/{:id}', component: () => import('./views/MemeEditInfo.vue'), name: 'meme.info_edit'},
  // { path: '/meme/edit/image/{:id}', component: () => import('./views/MemeEditImage.vue'), name: 'meme.image_edit' },
  // { path: '/meme/edit/complate/{:id}', component: () => import('./views/MemeFillInfo.vue'), name: 'meme.fill' },
  // Settings
  { path : '/settings', component: () => import('./views/settings/Main.vue'), name: 'settings' },
  { path : '/settings/about', component: () => import('./views/settings/About.vue'), name: 'settings.about' },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})
export default router