import React, { useEffect } from 'react'
import ReactDOM from 'react-dom/client'

import { Suspense } from 'react'
import { RouterProvider } from 'react-router-dom'

import './i18n'
import router from './routes'
import LoadingPage from './pages/loading/page'
import './global.scss'
import { ThemeProvider, createTheme } from '@mui/material'
import { useSettings } from './store/settings'
import { useTranslation } from 'react-i18next'

function Root() {
  const themeName = useSettings((settings) => settings.theme) as 'light' | 'dark'
  const theme = createTheme({
    palette: {
      mode: themeName,
    },
  })

  const lang = useSettings(s => s.language)
  const { i18n } = useTranslation()
  useEffect(() => {
    i18n.changeLanguage(lang).catch((e) => console.error(e))
  }, [lang])

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
