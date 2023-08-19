import { dialog } from '@tauri-apps/api'

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
    return {files: null}
  } else {
    if (typeof value == 'object') {
      return {files: value}
    } else {
      return {files: [value]}
    }
  }

}