import { lazy } from "react";

const Page = lazy(() => import("./index"));
import loader from './index-loader'

const SubPageProject = lazy(() => import("./project"));
const SubPageGlobalPrefs = lazy(()=>import("@/pages/global-pref"))

export default {
  path: "/",
  element: <Page />,
  loader: loader,
  children: [
    {
      path: "/",
      element: <SubPageProject />,
    },
    {
        path: '/prefs',
        element: <SubPageGlobalPrefs/>
    }
  ],
};
