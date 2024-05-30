import { Tag } from "@/components/tag-viewer";
import { invoke } from "@tauri-apps/api/core";
export const getDefaultStickyDataDir = () =>
  invoke<string>("get_default_sticky_dir");

export const createPictureSticky = (
  name: string,
  pkg: string,
  path: string,
  tags: Tag[],
  withExt: boolean = true
) =>
  invoke<void>("create_pic_sticky", {
    name,
    pkg,
    path,
    tags,
    withExt,
  });

export const createTextSticky = (content: string, pkg: string, tags: Tag[]) =>
  invoke<void>("create_text_sticky", { content, pkg, tags });

export const hasStickyFile = (path: string, withExt: boolean = true) =>
  invoke<boolean>("has_sticky_file", {
    path,
    withExt,
  });

export const searchPackage = (keyword: string) =>
  invoke<string[]>("search_package", { keyword });

export type StickyTY = "PIC" | "TEXT";

export type StickyThumb = StickyTextThumb | StickyImgThumb;

export interface StickyTextThumb {
  id: number;
  name: string;
  ty: "TEXT";
}

export interface StickyImgThumb {
  id: number;
  path: string;
  name: string;
  width?: number;
  height?: number;
  ty: "PIC";
}

export const searchSticky = (
  stmt: string,
  page: number = 0
): Promise<StickyThumb[]> =>
  invoke("search_sticky", {
    stmt,
    page,
  });

export const countSearchStickyPage = (stmt: string): Promise<number> =>
  invoke("count_search_sticky_page", { stmt });

export const searchTagNamespace = (prefix: string): Promise<string[]> =>
  invoke("search_tag_ns", { prefix });

export const searchTagValue = (
  ns: string,
  valuePrefix: string
): Promise<string[]> => invoke("search_tag_value", { ns, valuePrefix });

export const blacklistPath = (path: string): Promise<void> =>
  invoke("blacklist_path", { path });

export const isPathBlacklist = (path: string): Promise<boolean> =>
  invoke("is_path_blacklist", { path });

export type Sticky = StickyText | StickyImg;

export interface StickyImg {
  id: number;
  path: string;
  name: string;
  width?: number;
  height?: number;
  package: string;
  tags: Tag[];
  ty: "PIC";
}
export interface StickyText {
  id: number;
  name: string;
  package: string;
  tags: Tag[];
  ty: "PIC" | "TEXT";
}

export const getStickyById = (id: number): Promise<Sticky> =>
  invoke("get_sticky_by_id", { id });
