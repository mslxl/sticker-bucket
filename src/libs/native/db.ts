import { invoke } from '@tauri-apps/api'
import { Tag } from '../../model/meme'

export interface MemeToAdd {
  name: string,
  description: string,
  ty: string,
  content?: string,
  fav: boolean,
  tags: Tag[],
  pkg_id: number
}

export async function addMemeRecord(meme: MemeToAdd) {
  await invoke('add_meme_record', {
    item: meme
  })
}