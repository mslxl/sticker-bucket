import * as R from 'ramda'
import { forwardRef, useImperativeHandle, useMemo, useRef, useState } from 'react'
import { Tag, collectTag } from '../model/meme'
import { Box, TextField, IconButton, Grid, Stack, Paper, Chip, Snackbar, Alert } from '@mui/material'
import { KeyboardReturn as ReturnIcon, Lock as LockIcon } from '@mui/icons-material'

export interface TagEditorProp {
  defaultValue?: Tag[]
  onChange?: (tags: Tag[]) => void
}

export interface TagEditorRef {
  textField: HTMLInputElement,
  tags: Tag[],
  clear(): void,
  add(name: string, value: string): void,
  remove(name: string, value: string): void,
  toggleLock(name: string, value: string): void,
}
const TagEditor = forwardRef<TagEditorRef, TagEditorProp>(({ defaultValue, onChange }, ref) => {
  const [snackbarOpen, setSnackbarOpen] = useState(false)
  const [snackbarMsg, setSnackbarMsg] = useState('')
  const closeSnackbar = () => setSnackbarOpen(false)
  function openSnackbar(msg: string) {
    setSnackbarMsg(msg)
    setSnackbarOpen(true)
  }

  const textFieldTag = useRef<HTMLInputElement>()

  const [tags, setTags] = useState<Tag[]>(defaultValue || [])
  const [lockTag, setLockTag] = useState<Tag[]>([])
  const tagM = useMemo(() => {
    return collectTag(tags)
  }, [tags])

  function handleToggleTagLock(key: string, value: string) {
    const tag = { key, value }
    if (R.includes(tag, lockTag)) {
      setLockTag(R.reject(R.equals(tag)))
    } else {
      setLockTag((state) => [...state, tag])
    }
  }
  function deleteTag(key: string, value: string) {
    setTags(R.reject(R.equals({ key, value })))
  }
  function addTag(key: string, value: string): boolean {
    if (R.find(R.equals({ key, value }), tags)) {
      return false
    }
    setTags((state) => [...state, { key, value }])
    return true
  }

  function clearTag() {
    setTags(
      R.filter(t => R.find(R.equals(t), lockTag) != undefined)
    )
  }

  function handleAddTag() {
    const text = textFieldTag.current!.value
    const [key, value] = text.split(':', 2)
    const result = addTag(key, value)
    if (!result) {
      openSnackbar('Tag already exists!')
    } else {
      onChange && onChange([...tags, { key, value }])
    }
    textFieldTag.current!.value = ''
  }

  useImperativeHandle(ref, () => ({
    textField: textFieldTag.current!,
    tags: tags,
    clear: clearTag,
    add: addTag,
    toggleLock: handleToggleTagLock,
    remove: deleteTag,
  }))

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

  return (
    <>
      <Box>
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
})
TagEditor.displayName = 'TagEditor'


export default TagEditor