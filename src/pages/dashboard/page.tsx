import { SpeedDial, SpeedDialAction, SpeedDialIcon } from '@mui/material'
import {
  Computer as ComputerIcon,
  TextFields as TextIcon
} from '@mui/icons-material'
import { useNavigate } from 'react-router-dom'
import { useDocumentTitle } from '../../libs/native/windows'


export default function DashboardPage() {
  const navigate = useNavigate()
  useDocumentTitle('Meme Management Dashboard')
  return (
    <>
      <h3>Saluton, la mondon</h3>
      <SpeedDial
        ariaLabel='Add Meme'
        sx={{ position: 'absolute', bottom: 16, right: 16 }}
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