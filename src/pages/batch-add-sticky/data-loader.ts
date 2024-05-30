import { imageExtensions } from "@/const";
import { isPathBlacklist } from "@/lib/cmd/library";
import { join } from "@tauri-apps/api/path";
import * as dialog from "@tauri-apps/plugin-dialog";
import { DirEntry, readDir } from "@tauri-apps/plugin-fs";
import { info } from "@tauri-apps/plugin-log";
import { any, filter, flatten, map } from "lodash/fp";

export interface BatchAddStickyLoaderData {
  images?: string[];
}

export async function listImageInFolder(path: string): Promise<string[]> {
  const entries = await readDir(path);
  return flatten(
    await Promise.all(
      map(
        (entry: DirEntry) => {
          if (entry.isDirectory && !entry.name.startsWith(".")) {
            return join(path, entry.name).then(listImageInFolder);
          } else {
            return join(path, entry.name);
          }
        },
        filter((entry: DirEntry) => {
          return (
            entry.isDirectory ||
            any((ext) => entry.name.endsWith(ext), imageExtensions)
          );
        }, entries)
      )
    )
  );
}

export async function batchAddStickyLoader(): Promise<BatchAddStickyLoaderData> {
  const folder = await dialog.open({
    title: "Add Sticky From Folder",
    directory: true,
  });
  if (!folder) {
    return {};
  }
  const images = await listImageInFolder(folder);

  const filteredImages = map(
    ([, img]:string) => img,
    filter(
      ([blacklisted]:any) => !blacklisted,
      await Promise.all(
        map(async (path) => {
          return [await isPathBlacklist(path), path];
        }, images)
      )
    )
  ) as any as string[];


  info(`Batch ${JSON.stringify(filteredImages)}`);
  return {
    images: filteredImages as any,
  };
}
