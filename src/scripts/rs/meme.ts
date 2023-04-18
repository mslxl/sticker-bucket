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

export async function openImageAndInterfer(): Promise<MemeInterfer | null> {
  return await invoke('open_image_and_interfere', {}) as MemeInterfer | null
}

export async function addMemeToLib(file:string, summary: string, desc: string, tags: {namespace: string, value:string}[], removeAfterAdd: boolean, extraData?: string){
  console.log({
    file: file,
    summary: summary,
    desc: desc,
    tags: tags,
    removeAfterAdd: removeAfterAdd,
    extraData: extraData
  })
  await invoke('add_meme', {
    file: file,
    summary: summary,
    desc: desc,
    tags: tags,
    removeAfterAdd: removeAfterAdd,
    extraData: extraData
  })
}