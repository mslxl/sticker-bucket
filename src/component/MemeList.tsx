import { MemeQueried } from '../libs/native/db'
import { useEffect, useRef, useMemo, useLayoutEffect } from 'react'
import { MemePreview } from './MemeListPreviewItem'
import Masonry from 'masonry-layout'

export interface MemeListProp {
  memes: MemeQueried[]
  hasNext: boolean,
  loadNextMeme: () => Promise<void>
}

export default function MemeList({ memes, hasNext, loadNextMeme }: MemeListProp) {
  const parentRef = useRef<HTMLDivElement>(null)

  const masonry = useRef<Masonry>()

  const listItems = useMemo(() => {
    return (
      memes.map((item) => (
        <MemePreview
          className='masonry-item'
          key={`meme-${item.id}`}
          meme={item}
          onLoad={() => masonry.current?.layout && masonry.current?.layout()} />
      ))
    )
  }, [memes])

  const loadInProgress = useRef(false)
  const scrollPosition = useRef(0)

  async function handleScroll() {
    scrollPosition.current = window.scrollY
    if (!hasNext) return
    if (loadInProgress.current) return
    const body = document.querySelector('body')!
    const height = body.scrollHeight

    if (height - window.scrollY < 1100) {
      try {
        loadInProgress.current = true
        await loadNextMeme()
      } finally {
        loadInProgress.current = false
      }
    }
  }

  useEffect(() => {
    window.addEventListener('scroll', handleScroll)
    return () => {
      window.removeEventListener('scroll', handleScroll)
    }
  }, [loadNextMeme])

  useLayoutEffect(() => {
    handleScroll()
    masonry.current = new Masonry(parentRef.current!, {
      itemSelector: '.masonry-item',
      gutter: 12,
    })

    window.scrollTo(0, scrollPosition.current)

    return () => {
      if (masonry.current && masonry.current.destroy) {
        masonry.current.destroy()
      }
    }
  }, [parentRef.current, memes])

  return (
    <div ref={parentRef}>
      {listItems}
    </div>
  )
}
