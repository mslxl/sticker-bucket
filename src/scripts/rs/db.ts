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

export interface MemeInterfer {
  path: string,
  summary: string | null,
  desc: string | null,
  tags: {
    namespace: string,
    value: string
  }[]
}

export interface Meme{
  id: number,
  content: string,
  extraData: string,
  summary: string,
  desc: string
}

export async function openImageAndInterfer(): Promise<MemeInterfer | null> {
  return await invoke('open_image_and_interfere', {}) as MemeInterfer | null
}

export async function addMemeToLib(file:string, summary: string, desc: string, tags: {namespace: string, value:string}[], removeAfterAdd: boolean, extraData?: string){
  await invoke('add_meme', {
    file: file,
    summary: summary,
    desc: desc,
    tags: tags,
    removeAfterAdd: removeAfterAdd,
    extraData: extraData
  })
}

export async function getMemeByPage(page: number) : Promise<Meme[]>{
  return invoke('query_all_memes_by_page', { page })
}

export async function getImageRealPath(imageId: string) : Promise<string>{
  return invoke('get_image_real_path', { imageId })
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