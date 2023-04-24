import { invoke } from '@tauri-apps/api'

export function isDebug(): Promise<boolean> {
  return invoke('is_debug', {})
}