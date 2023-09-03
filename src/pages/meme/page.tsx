import { useLoaderData } from 'react-router-dom'
import { MemeLoadValue } from './loader'
import { useDocumentTitle } from '../../libs/native/windows'
import { AppBar, Box, Button, Card, CardActions, CardContent, Container, CssBaseline, IconButton, Toolbar, Typography } from '@mui/material'
import { ArrowBack as BackIcon } from '@mui/icons-material'
import { tauri } from '@tauri-apps/api'
import { useMemo } from 'react'
import { collectTag } from '../../model/meme'
import TagShowcase from '../../component/TagShowcase'

export default function MemePreviewPage() {
  const value = useLoaderData() as MemeLoadValue

  useDocumentTitle(value.meme.name)
  const tagM = useMemo(() => collectTag(value.tags), [value.tags])

  return (
    <Box sx={{ flexGrow: 1 }}>
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
      <Container>
        <Card variant='outlined'>
          <CardContent sx={{display: 'flex'}}>
            {
              value.meme.ty == 'image' ? (
                <Box
                  component="img"
                  src={tauri.convertFileSrc(value.meme.path)}
                  sx={{ margin: 2 }} />
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
            <Button size='small'>Fav</Button>
            <Button size='small'>Copy</Button>
            <Button size='small'>Edit</Button>
            <Button size='small'>Extend</Button>

          </CardActions>
        </Card>

      </Container>

    </Box>
  )

}