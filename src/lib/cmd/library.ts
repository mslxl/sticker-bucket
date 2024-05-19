import { Tag } from "@/components/tag-viewer";
import { invoke } from "@tauri-apps/api/core";
export const getDefaultStickyDataDir = () =>
  invoke<string>("get_default_sticky_dir");

export const createSticky = (
  name: string,
  pkg: string,
  path: string,
  tags: Tag[],
  withExt: boolean = true
) =>
  invoke<void>("create_sticky", {
    name,
    pkg,
    path,
    tags,
    withExt,
  });

export const hasStickyFile = (path: string, withExt: boolean = true) =>
  invoke<boolean>("has_sticky_file", {
    path,
    withExt,
  });

export const searchPackage = (keyword: string) =>
  invoke<string[]>("search_package", { keyword });

export interface StickyThumb {
  id: number;
  path: string;
  name: string;
  width?: number;
  height?: number;
}

export const searchSticky = (
  stmt: string,
  page: number = 0
): Promise<StickyThumb[]> =>
  invoke("search_sticky", {
    stmt,
    page,
  });
