import { Box, Container, CssBaseline, Grid, Paper, Stack, useMediaQuery } from '@mui/material'
import { Outlet } from 'react-router-dom'
import IconImage from './icon.png'

function Icon() {
  return (
    <Container>
      <Stack sx={{ alignItems: 'center' }}>
        <Box
          component="img"
          src={IconImage}
          sx={{
            maxWidth: '192px'
          }} />
        <h2 css={{ textAlign: 'center' }}>Meme Management</h2>
      </Stack>
    </Container>
  )
}


export default function WelcomeLayout() {
  const doubleColumn = useMediaQuery('(min-width:600px)')

  return (
    <>
      <CssBaseline />
      <Grid container>
        {
          doubleColumn && (
            <Grid item xs={4}>
              <Paper
                square
                elevation={6}>
                <Box sx={{
                  display: 'flex',
                  alignItems: 'center',
                  height: '100vh',
                }}>
                  <Icon />
                </Box>
              </Paper>
            </Grid>
          )
        }
        <Grid item xs={8}>
          <Container>
            <Outlet />
          </Container>
        </Grid>
      </Grid>
    </>
  )
}