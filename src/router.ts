import MainView from './views/Main.vue'

import MemeAddView from './views/MemeAdd.vue'

import {createRouter, createWebHashHistory} from 'vue-router'

const routes = [
  { path : '/', component: MainView, name: 'main' },
  { path : '/meme/add', component: MemeAddView, name: 'meme.add'}
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})
export default router