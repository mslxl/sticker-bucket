import { MemeQueried } from '../libs/native/db'
import { CircularProgress, ImageList, ImageListItem } from '@mui/material'
import { useEffect, useMemo, useRef, useState } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import { useWindowsSize } from '../libs/listener'
import { MemePreview } from './MemePreview'

export interface MemeListProp {
  memes: MemeQueried[]
  hasNext: boolean,
  loadNextMeme: () => Promise<void>
}

export default function MemeList({ memes, hasNext, loadNextMeme }: MemeListProp) {
  const parentRef = useRef<HTMLUListElement>(null)

  const listVirtualizer = useVirtualizer({
    count: hasNext ? memes.length + 1 : memes.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 100,
    overscan: 5
  })

  const fetchingNext = useRef(false)
  useEffect(() => {
    const [lastItem] = [...listVirtualizer.getVirtualItems()].reverse()
    if (!lastItem) return
    if (lastItem.index >= memes.length - 1 && hasNext && !fetchingNext.current) {
      fetchingNext.current = true
      loadNextMeme().then(() => {
        fetchingNext.current = false
      })
    }
  }, [hasNext, loadNextMeme, memes, listVirtualizer.getVirtualItems()])


  const listItems = useMemo(() => {
    return listVirtualizer.getVirtualItems().map((virtualItem) => {
      if (memes.length <= virtualItem.index) {
        return (
          <ImageListItem key={`load-${virtualItem.index}`}>
            <CircularProgress />
          </ImageListItem>
        )

      } else {
        const item = memes[virtualItem.index]
        return <MemePreview key={`meme-${item.id}`} meme={item} />
      }
    })
  }, [listVirtualizer.getVirtualItems()])



  const [cols, setCols] = useState(2)

  useWindowsSize(({ width }) => {
    if (parentRef.current) {
      const cols = Math.floor(width / 384)
      setCols(cols)
    }
  })

  return (
    <>
      <ImageList variant='masonry' ref={parentRef} cols={cols}>
        {listItems}
      </ImageList>
    </>
  )
}