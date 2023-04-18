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