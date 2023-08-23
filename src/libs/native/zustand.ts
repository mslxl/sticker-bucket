import { invoke } from '@tauri-apps/api'
import { StateStorage } from 'zustand/middleware'

async function native_set(name: string, value: string): Promise<void> {
  return invoke('zustand_set', { name, value })
}

async function native_get(name: string): Promise<string | null> {
  return invoke('zustand_get', { name })
}

async function native_del(name: string): Promise<void> {
  return invoke('zustand_del', { name })
}

export class ZustandNativeStorage implements StateStorage{
  getItem(name: string) :Promise<string | null> {
    return native_get(name)
  }
  setItem(name: string, value: string): Promise<void>{
    return native_set(name, value)
  }
  removeItem(name: string): Promise<void>{
    return native_del(name)
  }
}