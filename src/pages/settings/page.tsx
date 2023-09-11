import { Container, List} from '@mui/material'
import ComboBoxItem from '../../component/settings/ComboBoxItem'
import { useSettings } from '../../store/settings'

export default function SettingsPage() {
  const itemTheme = useSettings(s => s.theme)
  const setTheme = useSettings(s => s.setTheme)

  const itemLang = useSettings(s => s.language)
  const setLang = useSettings(s => s.setLanguage)


  return (
    <Container>
      <List sx={{ bgcolor: 'background.paper' }}>
        <ComboBoxItem
          title='Theme'
          option={[['Light', 'light'], ['Dark', 'dark']]}
          defaultValue={itemTheme}
          applyOption={setTheme} />
      </List>
    </Container>
  )
}