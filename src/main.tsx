import React, { Suspense, lazy } from "react";
import ReactDOM from "react-dom/client";
import { attachConsole } from "@tauri-apps/plugin-log";

import "normalize.css";
import "@/tailwind.css"
import "@/global.scss";
import { loadConst } from "./const";

Promise.all([attachConsole(), loadConst]).then(() => {
  const Router = lazy(() => import("@/router"));
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <Suspense>
        <Router />
      </Suspense>
    </React.StrictMode>
  );
});
