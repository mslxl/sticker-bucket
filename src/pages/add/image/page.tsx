import { tauri } from '@tauri-apps/api'
import { useLoaderData } from 'react-router-dom'
import { useDocumentTitle } from '../../../libs/native/windows'
import { Container, LinearProgress } from '@mui/material'
import MemeEditor from '../../../component/MemeEditor'
import { useState } from 'react'
import { Meme } from '../../../model/meme'
import { useDatabase } from '../../../store/database'

export default function AddPage() {
  const { files } = useLoaderData() as { files: string[] }
  if (files == null) {
    history.back()
  }
  const multifile = files.length != 1
  const [prog, setProg] = useState(0)

  useDocumentTitle(`Add image (${prog + 1}/${files.length})`)

  if (prog >= files.length) {
    // TODO: render message when multifile is true
    history.back()
  }

  const database = useDatabase()

  async function handleMemeAdd(meme: Meme) {
    let noError = true
    try {
      await database.addMeme({
        ...meme,
        pkg_id: 0,
        content: files[prog]
      })
    } catch (e) {
      noError = false
    }
    if (noError) {
      setProg(p => p + 1)
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