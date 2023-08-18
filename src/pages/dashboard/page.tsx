import { SpeedDial, SpeedDialAction, SpeedDialIcon } from '@mui/material'
import { Computer as ComputerIcon } from '@mui/icons-material'
import { useNavigate } from 'react-router-dom'


export default function DashboardPage() {
  const navigate = useNavigate()
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
          tooltipTitle="Local Source"
          onClick={() => navigate('/add')} />
      </SpeedDial>
    </>
  )
}