import { prefLngAtom } from "@/prefs";
import { useAtomValue } from "jotai";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";

export default function useLng(){
    const {i18n} = useTranslation()
    const lng = useAtomValue(prefLngAtom)
    useEffect(()=>{
        i18n.changeLanguage(lng)
    },[lng])
    
}