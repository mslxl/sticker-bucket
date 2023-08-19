import { useEffect } from 'react'
import { window } from '@tauri-apps/api'

export function useDocumentTitle(title: string) {
  function updateTitle(name: string) {
    document.title = name
    window.getCurrent().setTitle(name)
  }

  useEffect(() => {
    updateTitle(title)
    return () => {
      updateTitle('Untitled')
    }
  })
}