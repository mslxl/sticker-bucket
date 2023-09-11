import { fs, tauri } from '@tauri-apps/api'
import { useLoaderData, useNavigate } from 'react-router-dom'
import { useDocumentTitle } from '../../../libs/native/windows'
import { Container, LinearProgress } from '@mui/material'
import MemeEditor from '../../../component/MemeEditor'
import { useState } from 'react'
import { Meme } from '../../../model/meme'
import { addMemeRecord } from '../../../libs/native/db'
import { useTranslation } from 'react-i18next'

export default function AddPage() {
  const { files } = useLoaderData() as { files: string[] }
  const { t } = useTranslation()

  const multifile = files.length != 1
  const [prog, setProg] = useState(0)

  useDocumentTitle(t('Add Image Progress', { index: prog + 1, length: files.length }))

  const navigate = useNavigate()

  async function handleMemeAdd(meme: Meme, del: boolean) {
    let noError = true
    try {
      await addMemeRecord({
        ...meme,
        pkg_id: 0,
        content: files[prog]
      })
      if (del) {
        await fs.removeFile(files[prog])
      }
    } catch (e) {
      console.error(e)
      noError = false
    }
    if (noError) {
      if (prog + 1 >= files.length) {
        navigate(-1)
      } else {
        setProg(p => p + 1)
      }
    }
  }

  return (
    <>
      {multifile ? <LinearProgress variant="determinate" value={Math.round(prog / files.length * 100)} /> : <></>}
      <Container sx={{ padding: 2 }}>
        <MemeEditor
          confirm={handleMemeAdd}
          imageUrl={tauri.convertFileSrc(files[prog])} />
      </Container>
    </>
  )
}