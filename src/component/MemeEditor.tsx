import * as R from 'ramda'

import { Grid } from '@mui/material'
import { TextField } from '@mui/material'
import { IconButton } from '@mui/material'
import { Chip, Stack } from '@mui/material'
import { Alert, Box, Fab, Paper, Snackbar } from '@mui/material'
import { FormGroup, FormControlLabel, Checkbox } from '@mui/material'
import { KeyboardReturn as ReturnIcon, Lock as LockIcon, Done as DoneIcon } from '@mui/icons-material'
import { Meme, Tag, collectTag } from '../model/meme'
import { useMemo, useRef, useState } from 'react'

export interface MemeEditorProp {
  imageUrl: string
  defaultValue?: Meme
  confirm?: (meme: Meme) => Promise<void>
  cancel?: () => void
}

export default function MemeEditor({ imageUrl, defaultValue, confirm }: MemeEditorProp) {
  const [snackbarOpen, setSnackbarOpen] = useState(false)
  const [snackbarMsg, setSnackbarMsg] = useState('')
  const closeSnackbar = () => setSnackbarOpen(false)
  function openSnackbar(msg: string) {
    setSnackbarMsg(msg)
    setSnackbarOpen(true)
  }

  const [meme, setMeme] = useState(defaultValue || {
    id: null,
    name: '',
    description: '',
    tags: []
  } as Meme)
  const [lockTag, setLockTag] = useState<Tag[]>([])
  const tagM = useMemo(() => {
    return collectTag(meme.tags)
  }, [meme.tags])

  function handleToggleTagLock(key: string, value: string) {
    const tag = { key, value }
    if (R.includes(tag, lockTag)) {
      setLockTag(R.reject(R.equals(tag)))
    } else {
      setLockTag((state) => [...state, tag])
    }
  }
  function deleteTag(key: string, value: string) {
    setMeme((meme) => ({
      ...meme,
      tags: R.reject(R.equals({ key, value }), meme.tags)
    }))
  }
  function addTag(key: string, value: string): boolean {
    if (R.find(R.equals({ key, value }), meme.tags)) {
      return false
    }
    setMeme((meme) => {
      return {
        ...meme,
        tags: [...meme.tags, { key, value }]
      }
    })
    return true
  }

  const tagChipsStack = (
    tagM.map(namespace =>
      <Stack key={namespace.key} spacing={1} direction="row">
        <Chip label={namespace.key} variant='outlined' />
        {
          namespace.value.map((value) => ({
            lock: R.includes({ key: namespace.key, value }, lockTag),
            tag: { key: namespace.key, value },
          })).map(({ lock, tag }) =>
            <Chip
              key={tag.value}
              label={tag.value}
              avatar={lock ? <LockIcon /> : undefined}
              clickable
              onClick={() => handleToggleTagLock(tag.key, tag.value)}
              onDelete={lock ? undefined : (() => deleteTag(tag.key, tag.value))} />
          )
        }
      </Stack>
    )
  )

  const textFieldTag = useRef<HTMLInputElement>()
  function handleAddTag() {
    const text = textFieldTag.current!.value
    const [key, value] = text.split(':', 2)
    const result = addTag(key, value)
    if (!result) {
      openSnackbar('Tag already exists!')
    }
    textFieldTag.current!.value = ''
  }

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
          <FormControlLabel control={<Checkbox defaultChecked />} label='Favourite' />
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
        <Box
          sx={{
            width: '100%',
            display: 'flex'
          }}>
          <TextField inputRef={textFieldTag} label='Tag' variant='filled' sx={{ flex: '1' }} />
          <IconButton sx={{ p: '10px' }} onClick={handleAddTag}>
            <ReturnIcon />
          </IconButton>
        </Box>
        <Grid
          container
          spacing={2}>
          <Grid item xs={8}>
            <Paper
              variant='outlined'
              sx={{ padding: '6px' }}>
              <Stack spacing={2}>
                {
                  tagChipsStack
                }
              </Stack>
            </Paper>
          </Grid>
          <Grid item xs={4}>

          </Grid>
        </Grid>
      </Paper>
      <Box
        sx={{
          margin: '16px',
          display: 'flex',
          flexDirection: 'row-reverse'
        }}>
        <Fab
          onClick={()=>confirm && confirm(meme)}
          variant='extended'
          color='primary'>
          <DoneIcon sx={{ mr: 1 }} />
          Done
        </Fab>
      </Box>

      <Snackbar
        open={snackbarOpen}
        autoHideDuration={3000}
        onClose={closeSnackbar}>
        <Alert severity='warning'>
          {snackbarMsg}
        </Alert>
      </Snackbar>
    </>
  )
}