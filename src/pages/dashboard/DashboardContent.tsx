import { SpeedDial, SpeedDialAction, SpeedDialIcon } from '@mui/material'
import {
  Computer as ComputerIcon,
  TextFields as TextIcon
} from '@mui/icons-material'
import { useNavigate } from 'react-router-dom'
import { useDocumentTitle } from '../../libs/native/windows'
import MemeList from '../../component/MemeList'
import { useEffect, useState, useCallback } from 'react'
import { SearchResponse } from '../../libs/search'
import { useTranslation } from 'react-i18next'

export interface DashboardContent {
  initialSearchResponse: SearchResponse
}

export default function DashboardContent(props: DashboardContent) {
  const [response, setResponse] = useState(props.initialSearchResponse)
  const { t } = useTranslation()

  useEffect(() => {
    setResponse(props.initialSearchResponse)
  }, [props.initialSearchResponse])


  const loadNextResponseChunk = useCallback(async () => {
    if (response.hasNext()) {
      const nxt = await response.next()
      if (nxt.index > response.index) {
        setResponse(nxt)
      }
    }
  }, [response])

  const navigate = useNavigate()

  useDocumentTitle('Meme Management Dashboard')

  return (
    <>
      <MemeList
        memes={response.data()}
        hasNext={response.hasNext()}
        loadNextMeme={loadNextResponseChunk} />

      <SpeedDial
        ariaLabel={t('Add Sticky')}
        sx={{ position: 'fixed', bottom: 16, right: 16 }}
        icon={<SpeedDialIcon />}>
        <SpeedDialAction
          key='local'
          icon={<ComputerIcon />}
          tooltipTitle={t('Local Image Source')}
          onClick={() => navigate('/add/image')} />
        <SpeedDialAction
          key='text'
          icon={<TextIcon />}
          tooltipTitle={t('Local Text Source')}
          onClick={() => navigate('/add/text')} />
      </SpeedDial>
    </>
  )
}