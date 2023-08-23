import { SpeedDial, SpeedDialAction, SpeedDialIcon } from '@mui/material'
import {
  Computer as ComputerIcon,
  TextFields as TextIcon
} from '@mui/icons-material'
import { useNavigate } from 'react-router-dom'
import { useDocumentTitle } from '../../libs/native/windows'
import MemeList from '../../component/MemeList'
import { useDatabase } from '../../store/database'


export default function DashboardPage() {
  const navigate = useNavigate()
  const memes = useDatabase((state) => state.memes)
  const hasNext = useDatabase((state) => state.hasNext)
  const loadNextMemes = useDatabase((state) => state.loadNext)

  useDocumentTitle('Meme Management Dashboard')


  return (
    <>
      <MemeList
        memes={memes}
        hasNext={hasNext}
        loadNextMeme={loadNextMemes} />

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