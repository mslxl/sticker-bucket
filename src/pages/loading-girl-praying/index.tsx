import { useTranslation } from "react-i18next";

export default function LoadingGirlPraying() {
  const { t } = useTranslation();
  return (
    <div className="w-screen h-screen">
      <div className="absolute right-0 bottom-0 m-8">
        <h3 className="font-semibold text-lg">{t("i18n.common.loading")}</h3>
      </div>
    </div>
  );
}
