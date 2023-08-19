import { lazy } from 'react'
import { createBrowserRouter } from 'react-router-dom'

const DashboardLayout = lazy(() => import('./pages/dashboard/layout'))
const DashboardPage = lazy(() => import('./pages/dashboard/page'))

const AddLayout = lazy(() => import('./pages/add/layout'))
const AddImagePage = lazy(() => import('./pages/add/image/page'))
import addImagePageLoader from './pages/add/image/loader'
const AddTextPage = lazy(()=>import('./pages/add/text/page'))

const routes = createBrowserRouter([
  {
    path: '/',
    element: <DashboardLayout />,
    children: [
      {
        path: '/',
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