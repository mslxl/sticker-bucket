import { invoke } from '@tauri-apps/api'
import { MemePkg, Tag } from '../../model/meme'

export interface MemeToAdd {
  name: string,
  description?: string,
  ty: string,
  content: string,
  fav: boolean,
  tags: Tag[],
  pkg_id: number
}

export async function openStorage(path: string) {
  return await invoke('open_storage', {
    path
  })
}

export async function getStorage(): Promise<string> {
  return invoke('get_storage')
}

export async function addMemeRecord(meme: MemeToAdd) {
  await invoke('add_meme_record', {
    item: meme
  })
}


export interface MemeQueried extends MemePkg {
  path: string
}

export async function searchMeme(stmt: string, page: number): Promise<MemeQueried[]> {
  return invoke<MemeQueried[]>('search_meme', { stmt, page })
}

export async function getMemeById(id: number): Promise<MemeQueried> {
  return invoke<MemeQueried>('get_meme_by_id', { id })
}

export async function getTagsById(id: number): Promise<Tag[]> {
  return invoke<Tag[]>('get_tags_by_id', { id })
}