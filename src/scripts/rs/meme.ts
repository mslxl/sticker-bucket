import { invoke } from '@tauri-apps/api'
export interface MemeInterfer {
  path: string,
  summary: string | null,
  desc: string | null,
  tags: {
    namespace: string,
    value: string
  }
}

export async function open_image_and_interfer(): Promise<MemeInterfer | null> {
  return await invoke('open_image_and_interfer', {}) as MemeInterfer | null
}