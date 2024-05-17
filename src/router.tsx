import { RouterProvider, createBrowserRouter } from "react-router-dom";
import { Suspense, lazy } from "react";

const PageMain = lazy(() => import("@/pages/main"));
const PageMainAll = lazy(() => import("@/pages/main/all"));
const PageMainSettings = lazy(() => import("@/pages/main/settings"));

import { ModalStackProvider } from "@/store/modal";
import useTheme from "./lib/hook/useTheme";

const router = createBrowserRouter([
  {
    path: "/",
    element: <PageMain />,
    children: [
      {
        path: "/",
        element: <PageMainAll />,
      },
      {
        path: "/settings",
        element: <PageMainSettings />,
      },
    ],
  },
]);

export default function Router() {
  useTheme()
  return (
    <Suspense>
      <ModalStackProvider>
        <RouterProvider router={router} />
      </ModalStackProvider>
    </Suspense>
  );
}
