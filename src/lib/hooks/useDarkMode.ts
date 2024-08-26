import { useAtomValue } from "jotai";
import { prefDarkModeAtom } from "@/prefs";
import { useEffect } from "react";

export function useDarkMode() {
  const enableDarkMode = useAtomValue(prefDarkModeAtom);
  useEffect(() => {
    const root = document.querySelector("html");
    if (enableDarkMode) {
      root?.classList.add("dark");
    } else {
      root?.classList.remove("dark");
    }
  }, [enableDarkMode]);
}
