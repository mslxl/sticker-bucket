import { useEffect, useState, useDeferredValue } from 'react'

interface Size{
  height: number,
  width: number
}
export function useWindowsSize(callback: (sz: Size)=>void){
  const [size, setSize] = useState({height: 0, width: 0})
  const deferredSize = useDeferredValue(size)
  function listener(){
    setSize({
      height: window.innerHeight,
      width: window.innerWidth
    })
  }
  useEffect(()=>{
    window.addEventListener('resize', listener)
    return ()=>{
      window.removeEventListener('resize', listener)
    }
  })

  useEffect(()=>{
    callback(size)
  }, [deferredSize])

}