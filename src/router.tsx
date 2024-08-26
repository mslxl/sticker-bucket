import { Suspense } from "react";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import LoadingGirlPraying from "@/pages/loading-girl-praying";

import startupRouter from "@/pages/startup/router"

export default function Router() {

  const router = createBrowserRouter([
      startupRouter,
      {
        path: '/__internal/loading',
        element: <LoadingGirlPraying/>
      }
  ]);

  return (
    <Suspense fallback={<LoadingGirlPraying />}>
      <RouterProvider router={router} />
    </Suspense>
  );
}
