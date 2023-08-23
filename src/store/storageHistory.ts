import * as R from 'ramda'
import { create } from 'zustand'
import { combine, createJSONStorage, persist } from 'zustand/middleware'
import { ZustandNativeStorage } from '../libs/native/zustand'


export const useStorageHistory = create(
  persist(
    combine({ items: [] as string[] }, (set) => ({
      add(path: string) {
        set((state) => (
          { items: R.insert(0, path, R.reject(R.equals(path), state.items)) }
        ))
      },
      clear() {
        set(() => ({ items: [] }))
      },
      delete(path: string) {
        set((state) => ({ items: R.reject(R.equals(path), state.items) }))
      }
    })),
    {
      name: 'storage-history',
      storage: createJSONStorage(() => new ZustandNativeStorage())
    }
  )
)