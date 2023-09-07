import { MemeQueried, getMemeById, getTagsById } from '../../libs/native/db'
import { Tag } from '../../model/meme'

export interface MemeLoadValue {
  id: number,
  meme: MemeQueried,
  tags: Tag[]
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export default async function load({ params }: any): Promise<MemeLoadValue> {
  const id: number = parseInt(params.id)

  const meme = await getMemeById(id)
  const tags = await getTagsById(id)
  return {
    id,
    meme,
    tags
  }
}