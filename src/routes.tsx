import { lazy } from 'react'
import { createBrowserRouter } from 'react-router-dom'

const DashboardLayout = lazy(() => import('./pages/dashboard/layout'))
const DashboardPage = lazy(() => import('./pages/dashboard/page'))

const AddLayout = lazy(() => import('./pages/add/layout'))
const AddPage = lazy(() => import('./pages/add/page'))
import addPageLoader from './pages/add/loader'

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
        path: '/add/',
        element: <AddPage />,
        loader: addPageLoader
      }
    ]
  }
])

export default routes