import { AppBar, Box, CssBaseline, IconButton, Toolbar, Typography } from '@mui/material'
import { ArrowBack as BackIcon } from '@mui/icons-material'
import { AnimatePresence } from 'framer-motion'
import { Outlet } from 'react-router-dom'

export default function SettingsLayout() {
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
            Settings
          </Typography>
        </Toolbar>
      </AppBar>
      <CssBaseline />
      <Box component='main' sx={{ flex: 1, p: 3 }}>
        <AnimatePresence mode='wait'>
          <Outlet />
        </AnimatePresence>
      </Box>
    </Box>
  )

}