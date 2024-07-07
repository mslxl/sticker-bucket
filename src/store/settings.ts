import { DEFAULT_STICKY_DIR } from "@/const";
import { Store } from "@tauri-apps/plugin-store";
import { atomWithStorage } from "jotai/utils";
const store = new Store("stickerbucket.json");

class TauriStorage<T> {
  store: Store;
  constructor(store: Store) {
    this.store = store;
  }
  getItem(key: string, initialValue: T): PromiseLike<T> {
    return store.get<T>(key).then((v) => v ?? initialValue);
  }
  setItem(key: string, newValue: T): PromiseLike<void> {
    return store.set(key, newValue);
  }
  removeItem(key: string): PromiseLike<void> {
    return store.delete(key).then(() => void 0);
  }
  subscribe(key: string, callback: (value: T) => void, initialValue: T) {
    const unfn = store.onChange<T>((changedKey, value) => {
      if (key == changedKey) {
        if (value != null) {
          callback(value);
        } else {
          callback(initialValue);
        }
      }
    });
    return () => unfn.then((fn) => fn());
  }
}

const storage = new TauriStorage<string>(store);

function storageTyped<T>(): TauriStorage<T> {
  return storage as any;
}

const database = {
  stickerDir: atomWithStorage(
    "db.sticker-dir",
    DEFAULT_STICKY_DIR,
    storageTyped<string>()
  ),
};
const display = {
  darkMode: atomWithStorage(
    "display.dark",
    false,
    storageTyped<boolean>()
  ),
  hideNsfw: atomWithStorage(
    "display.hide-nsfw",
    true,
    storageTyped<boolean>()
  ),
  blurNsfw: atomWithStorage("display.blur-nsfw", true, storageTyped<boolean>()),
  nsfwTags: atomWithStorage(
    "display.nsfw-tags",
    ["misc:nsfw"],
    storageTyped<string[]>()
  ),
};

export default {
  database,
  display,
};
