import { RouterProvider, createBrowserRouter } from "react-router-dom";
import { Suspense, lazy } from "react";

const PageMain = lazy(() => import("@/pages/main"));
const PageMainAll = lazy(() => import("@/pages/main/all"));
const PageMainSettings = lazy(() => import("@/pages/main/settings"));

const PageAddFolder = lazy(() => import("@/pages/batch-add-sticky"));
import {batchAddStickyLoader} from "@/pages/batch-add-sticky/data-loader"

import { ModalStackProvider } from "@/store/modal";
import useTheme from "./lib/hook/useDarkMode";

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
  {
    path: "/add/folder",
    element: <PageAddFolder />,
    loader: batchAddStickyLoader
  },
]);

export default function Router() {
  useTheme();
  return (
    <Suspense>
      <ModalStackProvider>
        <RouterProvider router={router} />
      </ModalStackProvider>
    </Suspense>
  );
}
