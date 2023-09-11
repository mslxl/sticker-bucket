import { tauri } from '@tauri-apps/api'
import { useLoaderData, useNavigate } from 'react-router-dom'
import { useDocumentTitle } from '../../../libs/native/windows'
import { Container } from '@mui/material'
import MemeEditor from '../../../component/MemeEditor'
import { Meme } from '../../../model/meme'
import { MemeLoadValue } from '../loader'
import { updateMemeRecord } from '../../../libs/native/db'
import { useTranslation } from 'react-i18next'

export default function EditPage() {

  const value = useLoaderData() as MemeLoadValue
  const {t} = useTranslation()

  useDocumentTitle(t('Edit Name', {name: value.meme.name}))

  const meme: Meme = {
    ...value.meme,
    tags: value.tags
  }

  const navigate = useNavigate()

  // const database = useDatabase()

  async function handleMemeAdd(meme: Meme) {
    let noErr = true
    try {
      await updateMemeRecord(value.id, { ...meme, pkg_id: 0, content: '' })
    } catch(e) {
      noErr = false
      console.error(e)
    }
    if (noErr) {
      navigate(-1)
    }
  }

  return (
    <>
      <Container sx={{ padding: 2 }}>
        <MemeEditor
          confirm={handleMemeAdd}
          noDeleteCheckbox
          defaultValue={meme}
          imageUrl={tauri.convertFileSrc(value.meme.path)} />
      </Container>
    </>
  )
}