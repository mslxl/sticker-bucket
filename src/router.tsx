import { RouterProvider, createBrowserRouter } from "react-router-dom";
import { Suspense, lazy } from "react";

const PageMain = lazy(() => import("@/pages/main"));
const PageStickyListViewer = lazy(() => import("@/pages/main/viewer"));
const PageMainSettings = lazy(() => import("@/pages/main/settings"));

const PageAddFolder = lazy(() => import("@/pages/batch-add-sticky"));
import { batchAddStickyLoader } from "@/pages/batch-add-sticky/data-loader";

const PageViewer = lazy(() => import("@/pages/viewer"));
import stickyDataLoader from "./pages/viewer/stickyDataloader";

const PageError = lazy(() => import("@/pages/error"));

import { ModalStackProvider } from "@/store/modal";
import useTheme from "./lib/hook/useDarkMode";

const router = createBrowserRouter([
  {
    path: "/",
    element: <PageMain />,
    children: [
      {
        path: "/",
        element: <PageStickyListViewer />,
      },
      {
        path: "/list/:page",
        element: <PageStickyListViewer />,
      },
      {
        path: "/list/:stmt/:page",
        element: <PageStickyListViewer />,
      },
      {
        path: "/settings",
        element: <PageMainSettings />,
      },
    ],
    errorElement: <PageError />,
  },
  {
    path: "/add/folder",
    element: <PageAddFolder />,
    loader: batchAddStickyLoader,
    errorElement: <PageError />,
  },
  {
    path: "/viewer/:stickyId",
    element: <PageViewer />,
    loader: stickyDataLoader as any,
    errorElement: <PageError />,
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
