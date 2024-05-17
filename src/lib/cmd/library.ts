import { invoke } from "@tauri-apps/api/core";
export const getDefaultStickyDataDir = () =>
  invoke<string>("get_default_sticky_dir");
