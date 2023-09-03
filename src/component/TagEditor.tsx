import * as R from 'ramda'
import * as db from '../libs/native/db'
import { Fragment, forwardRef, useImperativeHandle, useMemo, useRef, useState } from 'react'
import { Tag, collectTag } from '../model/meme'
import { Box, TextField, IconButton, Grid, Stack, Paper, Snackbar, Alert, Autocomplete, CircularProgress } from '@mui/material'
import { KeyboardReturn as ReturnIcon } from '@mui/icons-material'
import TagShowcase from './TagShowcase'

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

  const tagShowcase = (<TagShowcase
    tags={tagM}
    lockTag={lockTag}
    lockable
    handleLock={handleToggleTagLock}
    handleDelete={deleteTag} />
  )
  const loading = false
  const [autocompleteOpen, setAutocompleteOpen] = useState(false)
  const [options, setOptions] = useState<Tag[]>([])

  async function updateAutocomplete() {
    const inp = textFieldTag.current?.value || ''
    if (inp.length == 0) {
      setOptions([])
    } else {
      if (inp.indexOf(':') == -1) {
        const opts = R.concat(
          R.map(k => ({ key: k, value: '' }), await db.getTagKeysByPrefix(inp)),
          await db.getTagsFuzzy(inp)
        )
        console.log(opts)
        setOptions(opts)
      } else {
        const [key, value] = inp.split(':', 2)
        const opts = await db.getTagsByPrefix(key, value)
        setOptions(opts)
      }
    }
  }

  return (
    <>
      <Box>
        <Box
          sx={{
            width: '100%',
            display: 'flex'
          }}>

          <Autocomplete
            sx={{ flex: '1' }}
            open={autocompleteOpen}
            onOpen={() => { setAutocompleteOpen(true); updateAutocomplete() }}
            onClose={() => setAutocompleteOpen(false)}
            loading={loading}
            options={options}
            filterOptions={R.identity}
            isOptionEqualToValue={R.identical}
            getOptionLabel={(option) => `${option.key}:${option.value}`}
            includeInputInList
            autoComplete
            renderInput={(params) => (
              <TextField
                {...params}
                inputRef={textFieldTag}
                label='Tag'
                variant='filled'
                onChange={updateAutocomplete}
                InputProps={{
                  ...params.InputProps,
                  endAdornment: (
                    <Fragment>
                      {loading ? <CircularProgress color='inherit' size={20} /> : null}
                      {params.InputProps.endAdornment}
                    </Fragment>
                  )
                }}
              />
            )}
          />

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
                  tagShowcase
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