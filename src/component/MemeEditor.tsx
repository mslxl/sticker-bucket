
import { Grid } from '@mui/material'
import { TextField } from '@mui/material'
import { Box, Fab, Paper } from '@mui/material'
import { FormGroup, FormControlLabel, Checkbox } from '@mui/material'
import { Done as DoneIcon } from '@mui/icons-material'
import { Meme, Tag } from '../model/meme'
import { useRef, useState } from 'react'
import TagEditor, { TagEditorRef } from './TagEditor'

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
      <Grid
        container
        spacing={2}
        sx={{
          minHeight: '100%'
        }}>
        <Grid item xs={6}>
          <Box
            component="img"
            src={imageUrl}
            sx={{ width: '100%' }} />
        </Grid>
        <Grid item xs={6}>
        </Grid>
      </Grid>
      <Paper sx={{ padding: 2 }}>
        <FormGroup>
          <FormControlLabel control={<Checkbox defaultChecked />} label='Delete file after add' />
          <FormControlLabel control={<Checkbox defaultChecked onChange={e => handleMemeFav(e.target.checked)} />} label='Favourite' />
        </FormGroup>
        <TextField
          fullWidth
          label='Name'
          variant='filled'
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