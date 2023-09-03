import { Backdrop, Box, Button, Divider, Fade, IconButton, ImageListItem, ImageListItemBar, ListItemIcon, ListItemText, Menu, MenuItem, MenuList, Modal, Paper, Typography } from '@mui/material'
import { MemeQueried } from '../libs/native/db'
import { Menu as MenuIcon, ContentCopy as CopyIcon, Message as DetailsIcon, Delete as DeleteIcon } from '@mui/icons-material'
import { tauri } from '@tauri-apps/api'
import { useNavigate } from 'react-router-dom'
import { useCallback, useRef, useState } from 'react'
import Image from './Image'
interface PreviewProps {
  meme: MemeQueried
  className?: string
  onLoad?: () => void
}
const dialogStyle = {
  position: 'absolute',
  top: '50%',
  left: '50%',
  transform: 'translate(-50%, -50%)',
  width: 400,
  bgcolor: 'background.paper',
  border: '2px solid #000',
  boxShadow: 24,
  p: 4,
}

function ImageMeme({ meme, onLoad }: PreviewProps) {
  return (
    <Image
      src={tauri.convertFileSrc(meme.path)}
      onLoad={onLoad} />
  )
}

function TextMeme({ meme }: PreviewProps) {
  return (
    <Typography
      variant="body1"
      gutterBottom
      sx={(theme)=>({
        minHeight: '120px',
        padding: '12px',
        maxHeight: '24em',
        color: theme.palette.common.black,
        overflow: 'hidden',
        textOverflow: 'ellipsis'
      })}>
      {meme.description}
    </Typography>
  )
}

function Content({ meme, onLoad }: PreviewProps) {
  if (meme.ty == 'image') {
    return <ImageMeme meme={meme} onLoad={onLoad} />
  } else {
    return <TextMeme meme={meme} />
  }
}

export function MemePreview({ meme, className, onLoad }: PreviewProps) {

  const anchor = useRef<HTMLButtonElement>(null)
  const [menuOpen, setMenuOpen] = useState(false)
  const handleMenuClose = useCallback(() => setMenuOpen(false), [setMenuOpen])
  const handleMenuShow = useCallback(() => setMenuOpen(true), [setMenuOpen])

  const navigateTo = useNavigate()
  function navigateToPreviewPage() {
    navigateTo(`/preview/${meme.id}`)
  }

  const [showDelDialog, setShowDelDialog] = useState(false)


  return (
    <>
      <Modal
        open={showDelDialog}
        onClose={() => setShowDelDialog(false)}
        closeAfterTransition
        slots={{ backdrop: Backdrop }}
        slotProps={{
          backdrop: {
            timeout: 500,
          }
        }}>
        <Fade in={showDelDialog}>
          <Box sx={dialogStyle}>
            <Typography variant='h6' component="h2">
              Delete Sticky?
            </Typography>
            <Typography sx={{ mt: 2 }} >
              Are you sure to delete {meme.name}
            </Typography>
          </Box>
        </Fade>
      </Modal>

      <ImageListItem
        className={className}
        sx={{
          maxWidth: '360px',
        }}>
        <Paper elevation={4}>
          <Button
            variant='text'
            onClick={navigateToPreviewPage}
            sx={{ width: '100%' }}>
            <Content meme={meme} onLoad={onLoad} />
          </Button>
          <ImageListItemBar
            title={meme.name}
            actionIcon={
              <IconButton
                ref={anchor}
                sx={{ color: 'rgba(255, 255, 255, 0.54)' }}
                aria-label={`info about ${meme.name}`}
                onClick={handleMenuShow}
              >
                <MenuIcon />
              </IconButton>}
          />
        </Paper>
        <Menu open={menuOpen} onClose={handleMenuClose} anchorEl={anchor.current}>
          <MenuList>
            <MenuItem onClick={handleMenuClose}>
              <ListItemIcon>
                <CopyIcon />
              </ListItemIcon>
              <ListItemText>Copy</ListItemText>
            </MenuItem>
            <MenuItem onClick={navigateToPreviewPage}>
              <ListItemIcon>
                <DetailsIcon />
              </ListItemIcon>
              <ListItemText>Detail</ListItemText>
            </MenuItem>
            <Divider />
            <MenuItem onClick={() => setShowDelDialog(true)}>
              <ListItemIcon>
                <DeleteIcon />
              </ListItemIcon>
              <ListItemText>Delete</ListItemText>
            </MenuItem>
          </MenuList>
        </Menu>
      </ImageListItem>
    </>
  )
}