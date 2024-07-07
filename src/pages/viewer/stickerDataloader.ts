import { getStickerById } from "@/lib/cmd/library"

export default async function stickerDataLoader ({params}: {params: {stickerId: string}}){
    const sticker = await getStickerById(parseInt(params.stickerId))
    console.log(sticker)
    return {
        sticker
    }
}