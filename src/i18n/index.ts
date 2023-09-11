import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'

import lngEnUS from './en_US'
import lngZhCN from './zh_CN'

i18n
  .use(initReactI18next)
  .init({
    resources: {
      en_US: { translation: lngEnUS },
      zh_CN: { translation: lngZhCN },
    },
    fallbackLng: 'en_US',
    interpolation: {
      escapeValue: false
    }
  })
