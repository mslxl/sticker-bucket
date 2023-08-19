import { create } from 'zustand'
import { combine } from 'zustand/middleware'
import { MemePkgFull } from '../model/meme'
import * as db from '../libs/native/db'
import { MemeToAdd } from '../libs/native/db'

export const useDatabase = create(
  combine(
    {
      loadedMemeIndex: -1,
      memes: [] as MemePkgFull[],
    },
    (set) => ({
      resetLoadedIndex() {
        set(() => (
          {
            loadedMemeIndex: -1,
            memes: []
          }
        ))
      },
      async addMeme (meme: MemeToAdd) {
        await db.addMemeRecord(meme)
        this.resetLoadedIndex()
      }
    })
  )
)
