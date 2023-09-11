import { create } from 'zustand'
import { combine, createJSONStorage, persist } from 'zustand/middleware'
import { ZustandNativeStorage } from '../libs/native/zustand'

export const useSettings = create(
  persist(
    combine(
      {
        theme: 'light',
        language: 'en_US'
      },
      (set) => ({
        setLanguage(lang: string) {
          set(() => ({ language: lang }))
        },
        setTheme(theme: string) {
          set({ theme })
        }
      })
    ),
    {
      name: 'settings',
      storage: createJSONStorage(() => new ZustandNativeStorage())
    }
  )
)
