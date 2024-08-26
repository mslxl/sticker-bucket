import { window } from "@tauri-apps/api";
import { useEffect } from "react";

export default function useTitle(title:string){
    useEffect(()=>{
        window.getCurrentWindow().setTitle(title)
        document.title = title
    }, [title])
}