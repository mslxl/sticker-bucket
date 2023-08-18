import { tauri } from '@tauri-apps/api'
import { useLoaderData } from 'react-router-dom'
import { useDocumentTitle } from '../../native/windows'
import { Container, LinearProgress } from '@mui/material'
import MemeEditor from '../../component/MemeEditor'
import { useState } from 'react'
import { Meme } from '../../model/meme'

export default function AddPage() {
  const { files } = useLoaderData() as { files: string[] }
  if (files == null) {
    history.back()
  }
  const multifile = files.length != 1
  const [prog, setProg] = useState(0)

  useDocumentTitle(`Add image (${prog + 1}/${files.length})`)

  async function handleMemeAdd(meme: Meme) {
    console.log(meme)
    setProg((state) => state + 1)
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