import { useAtomValue } from "jotai";
import cfg from "@/store/settings";
import { useEffect } from "react";

export default function useTheme() {
  const darkMode = useAtomValue(cfg.display.darkMode);
  const root = window.document.documentElement;
  useEffect(() => {
    if (darkMode) {
      root.classList.add("dark");
    } else {
      root.classList.add("light");
    }
    return () => root.classList.remove("light", "dark");
  });
}
