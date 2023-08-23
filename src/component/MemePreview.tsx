import { IconButton, ImageListItem, ImageListItemBar, Paper, Typography } from '@mui/material'
import { MemeQueried } from '../libs/native/db'
import { Info as InfoIcon } from '@mui/icons-material'
import { tauri } from '@tauri-apps/api'
interface PreviewProps {
  meme: MemeQueried
}

function ImageMeme({ meme }: PreviewProps) {
  return (
    <ImageListItem>
      <Paper elevation={4}>
        <img
          src={tauri.convertFileSrc(meme.path)}
          alt={meme.description}
          css={{
            minHeight: '120px',
          }}
          loading='lazy' />
      </Paper>
      <ImageListItemBar
        title={meme.name}
        subtitle={meme.description}
        actionIcon={
          <IconButton
            sx={{ color: 'rgba(255, 255, 255, 0.54)' }}
            aria-label={`info about ${meme.name}`}
          >
            <InfoIcon />
          </IconButton>}
      />
    </ImageListItem>
  )
}

function TextMeme({ meme }: PreviewProps) {
  return (
    <ImageListItem>
      <Paper elevation={4}>
        <Typography
          variant="body1"
          gutterBottom
          sx={{ minHeight: '120px', padding: '12px' }}>
          {meme.description}
        </Typography>
      </Paper>
      <ImageListItemBar
        title={meme.name}
        actionIcon={
          <IconButton
            sx={{ color: 'rgba(255, 255, 255, 0.54)' }}
            aria-label={`info about ${meme.name}`}
          >
            <InfoIcon />
          </IconButton>}
      />
    </ImageListItem>
  )
}

export function MemePreview({ meme }: PreviewProps) {
  if (meme.ty == 'image') {
    return <ImageMeme meme={meme} />
  } else {
    return <TextMeme meme={meme} />
  }

}