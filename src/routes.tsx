import { lazy } from 'react'
import { createBrowserRouter } from 'react-router-dom'

const WelcomeLayout = lazy(() => import('./pages/welcome/layout'))
const WelcomePage = lazy(() => import('./pages/welcome/page'))

const DashboardLayout = lazy(() => import('./pages/dashboard/layout'))
const DashboardPage = lazy(() => import('./pages/dashboard/page'))

const AddLayout = lazy(() => import('./pages/add/layout'))
const AddImagePage = lazy(() => import('./pages/add/image/page'))
import addImagePageLoader from './pages/add/image/loader'
const AddTextPage = lazy(() => import('./pages/add/text/page'))

const routes = createBrowserRouter([
  {
    path: '/',
    element: <WelcomeLayout />,
    children: [{ path: '/', element: <WelcomePage /> }]
  },
  {
    path: '/dashboard',
    element: <DashboardLayout />,
    children: [
      {
        path: '/dashboard',
        element: <DashboardPage />
      }
    ]
  },
  {
    path: '/add',
    element: <AddLayout />,
    children: [
      {
        path: '/add/image',
        element: <AddImagePage />,
        loader: addImagePageLoader
      },
      {
        path: '/add/text',
        element: <AddTextPage />
      }
    ]
  }
])

export default routes