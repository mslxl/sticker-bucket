import { exitWithError } from "./lib/sys";
import { getDefaultStickyDataDir } from "./lib/cmd/library";
import * as z from 'zod'

export let DEFAULT_STICKY_DIR: string;
export const ZOD_TAG = z.string().regex(/.+?:.+?/, {
  message: 'Value is not a tag'
})

export const loadConst = new Promise((resolve, reject) => {
  Promise.all([getDefaultStickyDataDir().then((v) => (DEFAULT_STICKY_DIR = v))])
    .catch(reject)
    .then(resolve);
}).catch((reason) => exitWithError(-1, reason));
