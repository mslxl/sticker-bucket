import { Tag } from "@/components/tag-viewer";
import { invoke } from "@tauri-apps/api/core";
export const getDefaultStickerDataDir = () =>
  invoke<string>("get_default_sticker_dir");

export const createPictureSticker = (
  name: string,
  pkg: string,
  path: string,
  tags: Tag[],
  withExt: boolean = true
) =>
  invoke<void>("create_pic_sticker", {
    name,
    pkg,
    path,
    tags,
    withExt,
  });

export const createTextSticker = (content: string, pkg: string, tags: Tag[]) =>
  invoke<void>("create_text_sticker", { content, pkg, tags });

export const hasStickerFile = (path: string, withExt: boolean = true) =>
  invoke<boolean>("has_sticker_file", {
    path,
    withExt,
  });

export const searchPackage = (keyword: string) =>
  invoke<string[]>("search_package", { keyword });

export type StickerTY = "PIC" | "TEXT";

export type StickerThumb = StickerTextThumb | StickerImgThumb;

export interface StickerTextThumb {
  id: number;
  name: string;
  ty: "TEXT";
}

export interface StickerImgThumb {
  id: number;
  path: string;
  name: string;
  width?: number;
  height?: number;
  ty: "PIC";
}

export const searchSticker = (
  stmt: string,
  page: number = 0
): Promise<StickerThumb[]> =>
  invoke("search_sticker", {
    stmt,
    page,
  });

export const countSearchStickerPage = (stmt: string): Promise<number> =>
  invoke("count_search_sticker_page", { stmt });

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

export type Sticker = StickerText | StickerImg;

export interface StickerImg {
  id: number;
  path: string;
  name: string;
  width?: number;
  height?: number;
  package: string;
  tags: Tag[];
  ty: "PIC";
}
export interface StickerText {
  id: number;
  name: string;
  package: string;
  tags: Tag[];
  ty: "PIC" | "TEXT";
}

export const getStickerById = (id: number): Promise<Sticker> =>
  invoke("get_sticker_by_id", { id });
