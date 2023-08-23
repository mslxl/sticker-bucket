import { Box, Checkbox, Fab, FormControlLabel, FormGroup, Paper, TextField } from '@mui/material'
import { Done as DoneIcon } from '@mui/icons-material'
import TagEditor, { TagEditorRef } from './TagEditor'
import { Meme, Tag } from '../model/meme'
import { useState, useRef } from 'react'

export interface MemeTextEditorProp {
  defaultValue?: Meme
  confirm?: (meme: Meme) => Promise<void>
  cancel?: () => void
}
export default function MemeTextEditor({ defaultValue, confirm }: MemeTextEditorProp) {
  const [meme, setMeme] = useState(defaultValue || {
    id: null,
    ty: 'text',
    name: '',
    content: '',
    tags: [],
    fav: false,
  } as Meme)

  const tagEditor = useRef<TagEditorRef>(null)

  function handleMemeName(name: string) {
    setMeme((state) => ({
      ...state,
      name
    }))
  }
  function handleMemeContent(content: string) {
    setMeme((state) => ({
      ...state,
      content: content,
      description: content,
    }))
  }

  function handleMemeTags(tags: Tag[]) {
    setMeme((state) => ({
      ...state,
      tags
    }))
  }

  function handleMemeFav(fav: boolean) {
    setMeme((state) => ({
      ...state,
      fav
    }))
  }

  return (
    <>
      <Paper>
        <FormGroup>
          <FormControlLabel control={<Checkbox defaultChecked onChange={e => handleMemeFav(e.target.checked)} />} label='Favourite' />
        </FormGroup>
        <TextField
          fullWidth
          label='Name'
          variant='filled'
          onChange={(e) => handleMemeName(e.target.value)} />
        <TextField
          fullWidth
          multiline
          label='Content'
          variant='filled'
          onChange={(e) => handleMemeContent(e.target.value)} />
        <TagEditor ref={tagEditor} onChange={handleMemeTags}/>
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