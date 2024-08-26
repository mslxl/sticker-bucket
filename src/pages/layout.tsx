import { useDarkMode } from "@/lib/hooks/useDarkMode";
import { ReactNode, useEffect } from "react";
import { DevTools } from "jotai-devtools";
import "jotai-devtools/styles.css";
export default function BasicLayout({ children }: { children: ReactNode }) {
  useDarkMode();
  useEffect(() => {
    const preventEvent = (event: MouseEvent) => event.preventDefault();
    document.addEventListener("contextmenu", preventEvent);
    return () => document.removeEventListener("contextmenu", preventEvent);
  }, []);

  return (
    <>
      {children}
      <DevTools />
    </>
  );
}
