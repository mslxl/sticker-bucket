import { create } from 'zustand'
import { combine } from 'zustand/middleware'
import * as db from '../libs/native/db'
import { MemeQueried, MemeToAdd } from '../libs/native/db'


export const useDatabase = create(
  combine(
    {
      stmt: '',
      loadedPage: -1,
      hasNext: true,
      memes: [] as MemeQueried[],
    },
    (set, get) => ({
      resetLoadedIndex() {
        set(() => (
          {
            loadedPage: -1,
            memes: [],
            hasNext: true,
            stmt: '',
          }
        ))
      },
      setSearchStatment(stmt: string) {
        if (get().stmt != stmt) {
          set(() => ({ stmt: stmt, loadedPage: -1, memes: [], hasNext: true }))
        }
      },
      async loadNext() {
        const stmt = get().stmt
        const page = get().loadedPage
        const result = await db.searchMeme(stmt, page + 1, false, false)
        if (result.length == 0) {
          set(() => ({
            hasNext: false
          }))
        } else {
          set((state) => ({
            loadedPage: page + 1,
            memes: [...state.memes, ...result]
          }))
        }

      },
      async addMeme(meme: MemeToAdd) {
        await db.addMemeRecord(meme)
        set(() => ({ loadedPage: -1, memes: [] }))
      }
    })
  )
)
