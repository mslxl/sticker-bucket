import { VueQueryPlugin } from '@tanstack/vue-query'
import { window } from '@tauri-apps/api'
import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import { handleHotUpdate, routes } from 'vue-router/auto-routes'
import logo from '@/../src-tauri/icons/128x128.png'
import App from '@/App.vue'
import '@/styles.css'

import('@tauri-apps/plugin-log').then(log => log.attachConsole())

const iconHead = document.head.appendChild(document.createElement('link'))
iconHead.setAttribute('rel', 'icon')
iconHead.setAttribute('href', logo)

const router = createRouter({
  history: createWebHistory(),
  routes,
})
setTimeout(() => {
  try {
    createApp(App)
      .use(router)
      .use(VueQueryPlugin, {
        queryClientConfig: {
          defaultOptions: {
            queries: {
              experimental_prefetchInRender: true,
            },
          },
        },
      })
      .mount('#app')
  }
  finally {
    window.getCurrentWindow().show()
  }
}, 0)

if (import.meta.hot) {
  handleHotUpdate(router)
}
