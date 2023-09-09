import { SpeedDial, SpeedDialAction, SpeedDialIcon } from '@mui/material'
import {
  Computer as ComputerIcon,
  TextFields as TextIcon
} from '@mui/icons-material'
import { useNavigate } from 'react-router-dom'
import { useDocumentTitle } from '../../libs/native/windows'
import MemeList from '../../component/MemeList'
import { useEffect, useState } from 'react'
import { SearchResponse } from '../../libs/search'

export interface DashboardContent{
  initialSearchResponse: SearchResponse

}

export default function DashboardContent(props: DashboardContent) {
  const [response, setResponse] = useState(props.initialSearchResponse)

  useEffect(()=>{
    setResponse(props.initialSearchResponse)
  }, [props.initialSearchResponse])

  async function loadNextResponseChunk(){
    if(response.hasNext()){
      const nxt = await response.next()
      setResponse(nxt)
    }
  }

  const navigate = useNavigate()

  useDocumentTitle('Meme Management Dashboard')

  return (
    <>
      <MemeList
        memes={response.data()}
        hasNext={response.hasNext()}
        loadNextMeme={loadNextResponseChunk} />

      <SpeedDial
        ariaLabel='Add Meme'
        sx={{ position: 'fixed', bottom: 16, right: 16 }}
        icon={<SpeedDialIcon />}>
        <SpeedDialAction
          key="local"
          icon={<ComputerIcon />}
          tooltipTitle="Local Image Source"
          onClick={() => navigate('/add/image')} />
        <SpeedDialAction
          key="text"
          icon={<TextIcon />}
          tooltipTitle="Local Text Source"
          onClick={() => navigate('/add/text')} />
      </SpeedDial>
    </>
  )
}