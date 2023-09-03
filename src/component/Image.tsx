import { Container } from '@mui/material'

export interface ImageProps {
  src: string
  onLoad?: ()=>void
}
export default function Image({ src, onLoad }: ImageProps) {
  return (
    <Container
      sx={{
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