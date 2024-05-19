import { StickyThumb } from "@/lib/cmd/library";
import StickyGrid from "./sticky-grid";

export interface StickyListProps{
    stickies: StickyThumb[]
    page?: number
    onPageChange?: (page: number)=>void
}

export default function StickyList(props: StickyListProps){
    return StickyGrid(props)
}