import * as R from 'ramda'
import * as db from '../libs/native/db'
import { Fragment, forwardRef, useEffect, useImperativeHandle, useMemo, useState } from 'react'
import { Tag, collectTag } from '../model/meme'
import { Box, TextField, IconButton, Grid, Stack, Paper, Snackbar, Alert, Autocomplete, CircularProgress, List, ListItem, ListItemButton, ListItemText } from '@mui/material'
import { KeyboardReturn as ReturnIcon } from '@mui/icons-material'
import TagShowcase from './TagShowcase'
import { useTranslation } from 'react-i18next'

export interface TagEditorProp {
  defaultValue?: Tag[]
  onChange?: (tags: Tag[]) => void
}

export interface TagEditorRef {
  tags: Tag[],
  clear(): void,
  add(name: string, value: string): void,
  remove(name: string, value: string): void,
  toggleLock(name: string, value: string): void,
}
const TagEditor = forwardRef<TagEditorRef, TagEditorProp>(({ defaultValue, onChange }, ref) => {
  const {t} = useTranslation()
  const [snackbarOpen, setSnackbarOpen] = useState(false)
  const [snackbarMsg, setSnackbarMsg] = useState('')
  const closeSnackbar = () => setSnackbarOpen(false)
  function openSnackbar(msg: string) {
    setSnackbarMsg(msg)
    setSnackbarOpen(true)
  }

  const [textInputTag, setTextInputTag] = useState('')

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
    onChange && onChange([...tags, { key, value }])
    return true
  }

  function clearTag() {
    setTags(
      R.filter(t => R.find(R.equals(t), lockTag) != undefined)
    )
    onChange && onChange([])
  }

  function handleAddTag() {
    if (textInputTag.indexOf(':') == -1) {
      return
    }
    const [key, value] = textInputTag.split(':', 2)
    const result = addTag(key, value)
    setTextInputTag('')
    if (!result) {
      openSnackbar(t('Tag already exists'))
    } 
  }

  const [relatedTags, setRelatedTags] = useState<db.TagFreq[]>([])
  useEffect(() => {
    db.getTagsRelated(tags).then(setRelatedTags)
  }, [tags])

  useImperativeHandle(ref, () => ({
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
  const recommendTags = (
    <List sx={{maxHeight: '32rem', overflowY: 'auto'}}>
      {
        relatedTags.map((t) => (
          <ListItem key={`${t.key}:${t.value}`}>
            <ListItemButton onClick={() => addTag(t.key, t.value)}>
              <ListItemText
                primary={`${t.key}:${t.value}`}
                secondary={t.freq} />
            </ListItemButton>
          </ListItem>
        ))
      }
    </List>
  )

  const loading = false
  const [autocompleteOpen, setAutocompleteOpen] = useState(false)
  const [options, setOptions] = useState<Tag[]>([])

  async function updateAutocomplete() {
    if (textInputTag.length == 0) {
      setOptions([])
    } else {
      const beforeInput = textInputTag
      if (textInputTag.indexOf(':') == -1) {
        const opts = R.concat(
          R.map(k => ({ key: k, value: '' }), await db.getTagKeysByPrefix(textInputTag)),
          await db.getTagsFuzzy(textInputTag)
        )
        if (textInputTag == beforeInput)
          setOptions(opts)
      } else {
        const [key, value] = textInputTag.split(':', 2)
        const opts = await db.getTagsByPrefix(key, value)
        if (textInputTag == beforeInput)
          setOptions(opts)
      }
    }
  }
  useEffect(() => {
    updateAutocomplete()
  }, [textInputTag])

  return (
    <>
      <Box>
        <Grid
          container
          spacing={2}>
          <Grid item xs={8}>
            <Box
              sx={{
                width: '100%',
                display: 'flex'
              }}>

              <Autocomplete
                sx={{ flex: '1' }}
                open={autocompleteOpen}
                onOpen={() => setAutocompleteOpen(true)}
                onClose={() => setAutocompleteOpen(false)}
                openOnFocus
                inputValue={textInputTag}
                onInputChange={(_, val) => setTextInputTag(val)}
                loading={loading}
                options={options}
                freeSolo
                disableClearable
                filterOptions={R.identity}
                isOptionEqualToValue={R.identical}
                groupBy={(option) => R.isEmpty(option.value) ? '' : option.key}
                getOptionLabel={(option) => typeof option === 'string' ? option : `${option.key}:${option.value}`}
                renderInput={(params) => (
                  <TextField
                    {...params}
                    label={t('Tag')}
                    variant='filled'
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

            <Paper
              variant='outlined'
              sx={{ padding: '6px' }}>
              <Stack spacing={2}>
                {tagShowcase}
              </Stack>
            </Paper>
          </Grid>
          <Grid item xs={4}>
            {recommendTags}
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