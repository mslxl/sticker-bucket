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
  return invoke('open_image_and_interfere', {}) 
}

export async function interferImage(path: string): Promise<MemeInterfer|null> {
  return invoke('interfere_image', { path })
}

export async function openPicturesList():Promise<string[]>{
  return invoke('open_picture_list', {})
}