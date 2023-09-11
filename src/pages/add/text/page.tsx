import { useDocumentTitle } from '../../../libs/native/windows'
import { Container } from '@mui/material'
import MemeTextEditor from '../../../component/MemeTextEditor'
import { Meme } from '../../../model/meme'
import { addMemeRecord } from '../../../libs/native/db'
import { useTranslation } from 'react-i18next'

export default function AddPage() {
  const {t} = useTranslation()

  useDocumentTitle(t('Add Text Snippet'))


  async function confirmAdd(meme: Meme) {
    addMemeRecord({
      ...meme,
      pkg_id: 0
    })

    history.back()
  }

  return (
    <>
      <Container sx={{ padding: 2 }}>
        <MemeTextEditor confirm={confirmAdd} />
      </Container>
    </>
  )
}

