import React from 'react'
import ReactDOM from 'react-dom/client'

import { Suspense } from 'react'
import { RouterProvider } from 'react-router-dom'

import './i18n'
import router from './routes'
import LoadingPage from './pages/loading/page'
import './global.scss'
import { ThemeProvider, createTheme } from '@mui/material'
import { useSettings } from './store/settings'

function Root() {
  const themeName = useSettings((settings)=>settings.theme) as 'light' | 'dark'
  const theme = createTheme({
    palette: {
      mode: themeName,
    },
  })

  return (
    <ThemeProvider theme={theme}>
      <RouterProvider router={router} />
    </ThemeProvider>
  )
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <Suspense fallback={<LoadingPage />}>
      <Root />
    </Suspense>
  </React.StrictMode>,
)
