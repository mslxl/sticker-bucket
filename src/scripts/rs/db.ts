import { invoke } from '@tauri-apps/api'

export function getTableVersion(): Promise<number> {
  return invoke('get_table_version', {}) as Promise<number>;
}

export function getDatabaseDir(): Promise<string>{
  return invoke('get_database_dir', {}) as Promise<string>;
}