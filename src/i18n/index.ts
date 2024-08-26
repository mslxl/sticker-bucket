import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import zhHans from "./zhHans";
import en from "./en";

export type Lng = {
  [P in keyof typeof zhHans]: (typeof zhHans)[P];
};

i18n
  .use(initReactI18next)
  .init({
    resources: {
      zhHans: {
        translation: zhHans,
      },
      en: {
        translation: en,
      },
    },
    // debug:true,
    fallbackLng: "zhHans",
    interpolation: {
      escapeValue: false,
    },
  });
