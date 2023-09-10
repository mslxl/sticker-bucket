import * as R from 'ramda'
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
  return invoke('add_meme_record', {
    item: meme
  })
}

export async function updateMemeRecord(id: number, meme: MemeToAdd){
  return invoke('update_meme_record', {
    memeId: id,
    item: meme
  })
}

export async function deleteMemeRecord(id: number){
  return invoke('delete_meme_by_id', {
    id: id
  })
}

export async function setTrashMemeRecord(id: number, trash: boolean){
  return invoke('trash_meme_by_id', {
    trash: trash,
    id: id
  })
}

export interface MemeQueried extends MemePkg {
  path: string
}

export async function searchMeme(stmt: string, page: number, fav: boolean, trash: boolean): Promise<MemeQueried[]> {
  return invoke<MemeQueried[]>('search_meme', { stmt, page, fav, trash })
}

export async function getMemeById(id: number): Promise<MemeQueried> {
  return invoke<MemeQueried>('get_meme_by_id', { id })
}

export async function getTagsById(id: number): Promise<Tag[]> {
  return invoke<Tag[]>('get_tags_by_id', { id })
}

export async function getTagKeysByPrefix(prefix: string): Promise<string[]> {
  return invoke<string[]>('get_tag_keys_by_prefix', { prefix: prefix })
}

export async function getTagsByPrefix(key: string, prefix: string): Promise<Tag[]> {
  return invoke<Tag[]>('get_tags_by_prefix', { key: key, prefix: prefix })
}

export async function getTagsFuzzy(keyword: string): Promise<Tag[]> {
  return invoke<Tag[]>('get_tags_fuzzy', { keyword: keyword })
}

export interface TagFreq extends Tag{
  freq: number
}

export async function getTagsRelated(tags: Tag[]): Promise<TagFreq[]>{
  if(R.isEmpty(tags)) return []
  return invoke<TagFreq[]>('get_tags_related', {tags: tags})
}

export async function setMemeFav(id: number, fav:boolean): Promise<void>{
  return invoke('set_meme_fav', {id, fav})
}
export async function setMemeTrash(id:number, trash: boolean): Promise<void>{
  return invoke('set_meme_trash', {id, trash})
}