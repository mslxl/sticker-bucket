
import { Backdrop, Button, CircularProgress, Grid } from '@mui/material'
import { TextField } from '@mui/material'
import { Box, Fab, Paper } from '@mui/material'
import { FormGroup, FormControlLabel, Checkbox } from '@mui/material'
import { Done as DoneIcon } from '@mui/icons-material'
import { Meme, Tag } from '../model/meme'
import { useRef, useState } from 'react'
import TagEditor, { TagEditorRef } from './TagEditor'
import Image from './Image'
import * as ocr from '../libs/ocr'

export interface MemeEditorProp {
  imageUrl: string
  defaultValue?: Meme
  confirm?: (meme: Meme) => Promise<void>
  cancel?: () => void
}

export default function MemeEditor({ imageUrl, defaultValue, confirm }: MemeEditorProp) {

  const [meme, setMeme] = useState(defaultValue || {
    id: null,
    ty: 'image',
    name: '',
    description: '',
    tags: [],
    fav: false,
    content: '',
  } as Meme)

  const refName = useRef<HTMLInputElement>()
  const [backDropOpen, setBackDropOpen] = useState(false)

  async function ocrText() {
    setBackDropOpen(true)
    ocr.paddle.recognize(imageUrl).then((name) => {
      if (refName.current) {
        setMeme((state) => ({
          ...state,
          name: name
        }))
        refName.current.value = name
        setBackDropOpen(false)
      }
    })
  }



  const tagEditor = useRef<TagEditorRef>(null)

  function handleMemeName(name: string) {
    setMeme((state) => ({
      ...state,
      name
    }))
  }
  function handleMemeDescription(desc: string) {
    setMeme((state) => ({
      ...state,
      description: desc
    }))
  }

  function handleMemeFav(fav: boolean) {
    setMeme((state) => ({
      ...state,
      fav
    }))
  }

  function updateTags(tags: Tag[]) {
    setMeme((state) => ({
      ...state,
      tags
    }))
  }

  return (
    <>
      <Backdrop
        sx={{
          color: '#fff',
          zIndex: (theme) => theme.zIndex.drawer + 1
        }}
        open={backDropOpen}>
        <CircularProgress color='inherit' />
      </Backdrop>
      <Grid
        container
        spacing={2}
        sx={{
          minHeight: '100%'
        }}>
        <Grid item xs={6}>
          <Image src={imageUrl} />
        </Grid>
        <Grid item xs={6}>
        </Grid>
      </Grid>
      <Paper sx={{ padding: 2 }}>
        <FormGroup>
          <FormControlLabel control={<Checkbox defaultChecked />} label='Delete file after add' />
          <FormControlLabel control={<Checkbox defaultChecked onChange={e => handleMemeFav(e.target.checked)} />} label='Favourite' />
        </FormGroup>
        <Button variant='contained' onClick={ocrText} sx={{ marginBottom: '12px' }}>OCR Name</Button>
        <TextField
          fullWidth
          inputRef={refName}
          label='Name'
          variant='filled'
          InputLabelProps={{ shrink: true }}
          onChange={(e) => handleMemeName(e.target.value)}
          defaultValue={meme.name} />
        <TextField
          fullWidth
          multiline
          label='Description'
          variant='filled'
          onChange={(e) => handleMemeDescription(e.target.value)}
          defaultValue={meme.description} />

        <TagEditor onChange={updateTags} ref={tagEditor} />
      </Paper>
      <Box
        sx={{
          margin: '16px',
          display: 'flex',
          flexDirection: 'row-reverse'
        }}>
        <Fab
          onClick={() => confirm && confirm(meme)}
          variant='extended'
          color='primary'>
          <DoneIcon sx={{ mr: 1 }} />
          Done
        </Fab>
      </Box>
    </>
  )
}