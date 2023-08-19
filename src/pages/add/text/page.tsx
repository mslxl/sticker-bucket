import { useDocumentTitle } from '../../../libs/native/windows'
import { Container } from '@mui/material'
import MemeTextEditor from '../../../component/MemeTextEditor'
import { Meme } from '../../../model/meme'
import { useDatabase } from '../../../store/database'

export default function AddPage() {

  useDocumentTitle('Add Text Snippet')
  const database = useDatabase()


  async function confirmAdd(meme: Meme) {
    await database.addMeme({
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

