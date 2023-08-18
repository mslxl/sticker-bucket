import i18n from "i18next"
import { initReactI18next } from "react-i18next"

import lngEn from "./en"

i18n
  .use(initReactI18next)
  .init({
    resources: {
      en:{
        translation: lngEn
      }
    },
    fallbackLng: "en",
    interpolation: {
      escapeValue: false
    }
  })
