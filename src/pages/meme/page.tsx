import { useLoaderData, useNavigate } from 'react-router-dom'
import { MemeLoadValue } from './loader'
import { useDocumentTitle } from '../../libs/native/windows'
import { AppBar, Box, Button, Card, CardActions, CardContent, Container, CssBaseline, IconButton, Toolbar, Typography } from '@mui/material'
import { ArrowBack as BackIcon } from '@mui/icons-material'
import { tauri } from '@tauri-apps/api'
import { useMemo, useState } from 'react'
import { collectTag } from '../../model/meme'
import TagShowcase from '../../component/TagShowcase'
import Image from '../../component/Image'
import { AnimatePresence, motion } from 'framer-motion'

import { Favorite as FavIcon, FavoriteBorder as FavOutlineIcon } from '@mui/icons-material'
import { setMemeFav } from '../../libs/native/db'
import { useTranslation } from 'react-i18next'

export default function MemePreviewPage() {
  const {t} = useTranslation()
  const [value, setValue] = useState(useLoaderData() as MemeLoadValue)

  useDocumentTitle(value.meme.name)
  const tagM = useMemo(() => collectTag(value.tags), [value.tags])
  const navigate = useNavigate()

  function toggleFav(){
    setMemeFav(value.id, !value.meme.fav)
    setValue((v)=>({...v, meme: {...v.meme, fav: !v.meme.fav}}))
  }

  return (
    <AnimatePresence mode='wait'>
      <Box sx={{ flexGrow: 1 }} key='preview'>
        <AppBar position='static'>
          <Toolbar>
            <IconButton
              onClick={() => history.back()}
              size='large'
              edge='start'
              color='inherit'
              sx={{ mr: 2 }}>
              <BackIcon />
            </IconButton>
            <Typography variant='h6' component="div" sx={{ flexGrow: 1 }}>
              {value.meme.name}
            </Typography>
          </Toolbar>
        </AppBar>
        <CssBaseline />
        <motion.div
          initial={{ width: 0 }}
          animate={{ width: '100%' }}
          exit={{ x: window.innerWidth }}>
          <Container>
            <Card variant='outlined'>
              <CardContent sx={{ display: 'flex' }}>
                {
                  value.meme.ty == 'image' ? (
                    <Image src={tauri.convertFileSrc(value.meme.path)} sx={{ maxHeight: '50%' }} />
                  ) : (
                    <Typography
                      variant='body1'>
                      {value.meme.description}
                    </Typography>
                  )
                }
                <TagShowcase tags={tagM} />
              </CardContent>
              <CardActions>
                <Button size='small'>{t('Copy Image')}</Button>
                <Button size='small' onClick={() => value.meme.ty == 'image' ? navigate(`/edit/image/${value.id}`) : navigate(`/edit/text/${value.id}`)}>{t('Edit')}</Button>
                <Button size='small'>{t('Extend Image')}</Button>
                <Button size='small' variant='outlined' onClick={toggleFav}>{value.meme.fav ? <FavIcon /> : <FavOutlineIcon />}&nbsp;{t('Favorite')}</Button>
              </CardActions>
            </Card>

          </Container>
        </motion.div>


      </Box>
    </AnimatePresence>
  )

}