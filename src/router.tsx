import { RouterProvider, createBrowserRouter } from "react-router-dom";
import { Suspense, lazy } from "react";

const PageMain = lazy(() => import("@/pages/main"));
const PageStickerListViewer = lazy(() => import("@/pages/main/viewer"));
const PageMainSettings = lazy(() => import("@/pages/main/settings"));

const PageAddFolder = lazy(() => import("@/pages/batch-add-sticker"));
import { batchAddStickerLoader } from "@/pages/batch-add-sticker/data-loader";

const PageViewer = lazy(() => import("@/pages/viewer"));
import stickerDataLoader from "./pages/viewer/stickerDataloader";

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
        element: <PageStickerListViewer />,
      },
      {
        path: "/list/:page",
        element: <PageStickerListViewer />,
      },
      {
        path: "/list/:stmt/:page",
        element: <PageStickerListViewer />,
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
    loader: batchAddStickerLoader,
    errorElement: <PageError />,
  },
  {
    path: "/viewer/:stickerId",
    element: <PageViewer />,
    loader: stickerDataLoader as any,
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
