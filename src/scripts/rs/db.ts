import { invoke } from '@tauri-apps/api'

export function getTableVersion(): Promise<number> {
  return invoke('get_table_version', {}) as Promise<number>;
}

export function getDataDir(): Promise<string>{
  return invoke('get_data_dir', {}) as Promise<string>;
}

export function getSQLiteVersion(): Promise<string>{
  return invoke('get_sqlite_version', {}) as Promise<string>
}

export interface Meme{
  id: number,
  content: string,
  extraData: string,
  summary: string,
  desc: string
}

export interface Tag{
  namespace: string,
  value: string
}

export async function addMemeToLib(file:string, summary: string, desc: string, tags: Tag[], removeAfterAdd: boolean, extraData?: string){
  await invoke('add_meme', {
    file: file,
    summary: summary,
    desc: desc,
    tags: tags,
    removeAfterAdd: removeAfterAdd,
    extraData: extraData
  })
}

export async function updateMeme(id: number, summary?: string, desc?: string, tags?: Tag[]):Promise<void>{
  await invoke('update_meme', {
    id,
    summary,
    desc,
    tags
  })
}

export async function getMemeByPage(search_stmt: string, page: number) : Promise<Meme[]>{
  return invoke('search_memes_by_text', {stmt: search_stmt, page })
}

export async function getImageRealPath(basename : string) : Promise<string>{
  return invoke('get_image_real_path', { basename })
}

export async function queryTagValueWithPrefix(namespace: string, prefix: string): Promise<string[]> {
  return invoke('query_tag_value_with_prefix', { namespace, prefix })
}

export async function queryNamespaceWithPrefix(prefix: string): Promise<string[]> {
  return invoke('query_namespace_with_prefix', { prefix })
}

export async function getMemeByID(id: number): Promise<Meme> {
  return invoke('query_meme_by_id', {id})
}

export async function getTagByMemeID(id: number): Promise<{namespace: string, value: string}[]>{
  return invoke('query_tag_by_meme_id', {id})
}

export async function queryCountMemes(): Promise<number> {
  return invoke('query_count_memes', {})
}

export async function queryCountTags(): Promise<number>{
  return invoke('query_count_tags', {})
}