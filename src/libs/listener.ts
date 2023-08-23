import { useEffect } from 'react'

interface Size{
  height: number,
  width: number
}
export function useWindowsSize(callback: (sz: Size)=>void){
  function listener(){
    callback({
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

}