import { invoke } from '@tauri-apps/api'
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
  return invoke('get_meme_by_page', { page })
}

export async function getImageRealPath(imageId: string) : Promise<string>{
  return invoke('get_image_real_path', { imageId })
}