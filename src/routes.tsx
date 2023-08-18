import { lazy } from "react"
import { createBrowserRouter } from "react-router-dom"

const Landing = lazy(() => import("./pages/Landing"))

const routes = createBrowserRouter([
  {
    path: "/",
    element: <Landing />
  }
])

export default routes