import React from 'react'
import ReactDOM from 'react-dom/client'

import { Suspense } from 'react'
import { RouterProvider } from 'react-router-dom'

import './i18n'
import router from './routes'
import LoadingPage from './pages/loading/page'
import './global.scss'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <Suspense fallback={<LoadingPage />}>
      <RouterProvider router={router} />
    </Suspense>
  </React.StrictMode>,
)
