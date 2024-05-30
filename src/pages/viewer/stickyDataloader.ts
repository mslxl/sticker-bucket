import { getStickyById } from "@/lib/cmd/library"

export default async function stickyDataLoader ({params}: {params: {stickyId: string}}){
    const sticky = await getStickyById(parseInt(params.stickyId))
    console.log(sticky)
    return {
        sticky
    }
}