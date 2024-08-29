import "@/i18n";
import React from "react";
import ReactDOM from "react-dom/client";
import "@unocss/reset/tailwind-compat.css";
import "@/global.sass";
import "virtual:uno.css";
import Router from "@/router";
import BasicLayout from "./pages/layout";
import { isDebugMode } from "./prefs";

isDebugMode().then((debugMode) => {
  if (!debugMode) {
    const preventEvent = (event: MouseEvent) => event.preventDefault();
    document.addEventListener("contextmenu", preventEvent);
  }
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BasicLayout>
      <Router />
    </BasicLayout>
  </React.StrictMode>
);
