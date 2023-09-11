import { Container, List} from '@mui/material'
import ComboBoxItem from '../../component/settings/ComboBoxItem'
import { useSettings } from '../../store/settings'
import { useTranslation } from 'react-i18next'

export default function SettingsPage() {
  const {t} = useTranslation()

  const itemTheme = useSettings(s => s.theme)
  const setTheme = useSettings(s => s.setTheme)

  const itemLang = useSettings(s => s.language)
  const setLang = useSettings(s => s.setLanguage)


  return (
    <Container>
      <List sx={{ bgcolor: 'background.paper' }}>
        <ComboBoxItem
          title={t('Theme')}
          option={[[t('Light Theme'), 'light'], [t('Dark Theme'), 'dark']]}
          defaultValue={itemTheme}
          applyOption={setTheme} />
        <ComboBoxItem
          title={t('Language')}
          option={[['中文', 'zh_CN'],['English', 'en_US'], ['Esperanto', 'epo']]}
          defaultValue={itemLang}
          applyOption={setLang}/>
      </List>
    </Container>
  )
}