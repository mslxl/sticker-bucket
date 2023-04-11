import MainView from './views/Main.vue'

import {createRouter, createWebHashHistory} from 'vue-router'

const routes = [
  { path : '/', component: MainView, name: 'main' }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})
export default router