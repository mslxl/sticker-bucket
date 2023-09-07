import { create } from 'zustand'
import { combine, createJSONStorage, persist } from 'zustand/middleware'
import { ZustandNativeStorage } from '../libs/native/zustand'

export const useSettings = create(
  persist(
    combine(
      {
        language: 'en_US'
      },
      (set) => ({
        setLanguage(lang: 'en_US' | 'zh_CN' | 'epo'){
          set(()=>({language: lang}))
        }
      })
    ),
    {
      name: 'settings',
      storage: createJSONStorage(() => new ZustandNativeStorage())
    }
  )
)
