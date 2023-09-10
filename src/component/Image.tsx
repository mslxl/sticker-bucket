import { Container, SxProps, Theme } from '@mui/material'

export interface ImageProps {
  src: string
  sx?:  SxProps<Theme>
  onLoad?: ()=>void
}
export default function Image({ src, onLoad, sx }: ImageProps) {
  return (
    <Container
      sx={{
        ...sx,
        width: '100%',
      }}
    >
      <img
        src={src}
        onLoad={()=>onLoad && onLoad()}
        style={{
          objectFit: 'cover',
          width: '100%',
          height: '100%'
        }}/>

    </Container>
  )
}