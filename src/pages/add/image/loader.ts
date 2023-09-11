import { dialog } from '@tauri-apps/api'
import { redirect } from 'react-router-dom'

export default async function loader() {
  const value = await dialog.open({
    multiple: true,
    recursive: true,
    filters: [{
      name: 'Pictures',
      extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif']
    }]
  })

  if (value === null) {
    return redirect('/dashboard')
  } else {
    if (typeof value == 'object') {
      return { files: value }
    } else {
      return { files: [value] }
    }
  }

}