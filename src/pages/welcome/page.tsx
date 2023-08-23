import { path, dialog } from '@tauri-apps/api'
import { Fab, List, ListItem, ListItemButton, ListItemText, capitalize } from '@mui/material'
import { useStorageHistory } from '../../store/storageHistory'
import { Add as AddIcon } from '@mui/icons-material'
import { openStorage } from '../../libs/native/db'
import { useNavigate } from 'react-router-dom'

export default function WelcomePage() {
  const storageHistory = useStorageHistory()
  const navigate = useNavigate()

  function handleOpenStorage(path: string) {
    storageHistory.add(path)
    openStorage(path)
    navigate('/dashboard')
  }
  async function handleAddStorage() {
    const path = await dialog.open({
      multiple: false,
      directory: true
    })
    if (path) {
      handleOpenStorage(path as string)
    }
  }

  return (
    <>
      <List sx={{ width: '100%', bgcolor: 'background.paper' }}>
        {
          storageHistory.items.map((item) => (
            <>
              <ListItem key={item}>
                <ListItemButton onClick={() => handleOpenStorage(item)}>
                  <ListItemText
                    primary={capitalize(item.substring(item.lastIndexOf(path.sep) + 1))}
                    secondary={item} />
                </ListItemButton>
              </ListItem>
            </>
          ))
        }
      </List>
      <Fab
        sx={{ position: 'absolute', bottom: 16, right: 16 }}
        color="primary"
        aria-label="open"
        onClick={handleAddStorage}>
        <AddIcon />
      </Fab>
    </>
  )

}