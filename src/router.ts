import MainView from './views/Main.vue'

import MemeAddView from './views/MemeAdd.vue'
import SettingsView from './views/Settings.vue'

import {createRouter, createWebHashHistory} from 'vue-router'

const routes = [
  { path : '/', component: MainView, name: 'main' },
  { path : '/settings', component: SettingsView, name: 'settings' },
  { path : '/meme/add', component: MemeAddView, name: 'meme.add'}
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})
export default router