import { Box, AppBar, Toolbar, IconButton, Typography, CssBaseline } from '@mui/material'
import { ArrowBack as ArrowBackIcon } from '@mui/icons-material'
import { Outlet } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
export default function AddLayout() {
  const { t } = useTranslation()
  return (
    <Box
      sx={{
        flexGrow: 1,
        minHeight: '100vh'
      }}>
      <AppBar position="static">
        <Toolbar>
          <IconButton
            size="large"
            edge="start"
            color="inherit"
            aria-label="open drawer"
            sx={{ mr: 2 }}
            onClick={() => history.back()}
          >
            <ArrowBackIcon />
          </IconButton>
          <Typography
            variant="h6"
            noWrap
            component="div"
            sx={{ flexGrow: 1, display: { xs: 'none', sm: 'block' } }}>
            {t('Add Sticky')}
          </Typography>
        </Toolbar>
      </AppBar>
      <CssBaseline />
      <Outlet />
    </Box>
  )
}