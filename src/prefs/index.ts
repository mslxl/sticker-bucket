import { atom, WritableAtom } from "jotai/vanilla";
import { atomWithObservable } from "jotai/utils";

import { AppGlobalCfg, commands, events } from "./type";
import { filter, identical, negate, uniq } from "lodash/fp";

type Observer<T> = {
  next: (value: T) => void;
};

type Subscription = {
  unsubscribe: () => void;
};

const realtimePrefsAtom = atomWithObservable(() => ({
  subscribe: (observer: Observer<AppGlobalCfg>): Subscription => {
    async function pullData() {
      let data = await commands.getGlobalPrefs();
      if (data.status == "ok") {
        observer.next(data.data);
      } else {
        alert(data.error);
      }
    }
    let unlisten = events.globalPrefsChangedEvent.listen(async (_cb) => {
      pullData();
    });
    pullData();

    return {
      unsubscribe: () => unlisten.then((f) => f()),
    };
  },
}));

export const prefDarkModeAtom = atom((get)=>get(globalPrefsAtom).dark_mode)
export const prefLngAtom = atom((get)=>get(globalPrefsAtom).lng)

export const prefHistoryAtom = atom((get)=>get(globalPrefsAtom).history)
export const addPrefHistoryAtom = atom(null, (get, set, data: string)=>{
  const old = get(globalPrefsAtom)
  set(globalPrefsAtom, {
    ...old,
    history: uniq([data, ...(old.history ?? [])])
  })
})
export const removePrefHistoryAtom = atom(null, (get, set, data: string)=>{
  const old = get(globalPrefsAtom)
  set(globalPrefsAtom, {
    ...old,
    history: filter(negate(identical))(data)
  })
})



export const globalPrefsAtom: WritableAtom<AppGlobalCfg, [data: AppGlobalCfg], void> = atom(
  (get) => get(realtimePrefsAtom),
  (_get, _set, data:AppGlobalCfg) => {
    commands.setGlobalPrefs(data, "");
  }
) as any;

export const isDebugMode = commands.isDebugMode