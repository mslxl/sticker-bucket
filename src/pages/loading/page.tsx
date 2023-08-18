import { Container, CircularProgress, CssBaseline } from '@mui/material'
export default function LoadingPage() {
  return (
    <>
      <CssBaseline />
      <Container sx={{
        width: '100vw',
        height: '100vh',
      }}>
        <CircularProgress sx={{
          margin: 0,
          position: 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
        }}/>
      </Container>
    </>
  )
}