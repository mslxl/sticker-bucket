import { MemeQueried } from '../libs/native/db'
import { CircularProgress, ImageListItem } from '@mui/material'
import { useEffect, useMemo, useRef } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import { MemePreview } from './MemeListPreviewItem'
import Masonry from 'masonry-layout'
import { useWindowsSize } from '../libs/listener'

export interface MemeListProp {
  memes: MemeQueried[]
  hasNext: boolean,
  loadNextMeme: () => Promise<void>
}

export default function MemeList({ memes, hasNext, loadNextMeme }: MemeListProp) {
  const parentRef = useRef<HTMLDivElement>(null)

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

  const masonry = useRef<Masonry>()
  useEffect(() => {
    masonry.current = new Masonry(parentRef.current!, {
      itemSelector: '.masonry-item',
      gutter: 12,
      columnWidth: 350
      // fitWidth: true
    })
    return () => {
      if (masonry.current && masonry.current.destroy) {
        masonry.current.destroy()
      }
    }
  })

  useWindowsSize(()=>{
    masonry.current?.layout && masonry.current.layout()
  })

  const listItems = useMemo(() => {
    return listVirtualizer.getVirtualItems().map((virtualItem) => {
      if (memes.length <= virtualItem.index) {
        return (
          <ImageListItem className='masonry-item' key={`load-${virtualItem.index}`}>
            <CircularProgress />
          </ImageListItem>
        )
      } else {
        const item = memes[virtualItem.index]
        return <MemePreview
          className='masonry-item'
          key={`meme-${item.id}`}
          meme={item}
          onLoad={() => masonry.current?.layout && masonry.current?.layout()} />
      }
    })
  }, [listVirtualizer.getVirtualItems()])


  return (
    <>
      <div
        ref={parentRef}>
        {listItems}
      </div>
    </>
  )
}