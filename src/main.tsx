import React from "react"
import ReactDOM from "react-dom/client"

import { Suspense } from "react"
import { RouterProvider } from "react-router-dom"

import "./i18n"
import router from "./routes"

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Suspense fallback={<>Loading</>}>
      <RouterProvider router={router} />
    </Suspense>
  </React.StrictMode>,
)
