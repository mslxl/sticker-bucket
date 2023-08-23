import * as R from 'ramda'
export interface Tag {
  key: string,
  value: string,
}
export interface TagM {
  key: string,
  value: string[],
}
export interface Meme {
  id: string | null,
  ty: 'image' | 'text',
  name: string,
  description?: string,
  content: string,
  tags: Tag[],
  fav: boolean,
}

export interface MemePkg extends Meme {
  content: string,
  trash: string,

  pkg_id: number,
}

export interface MemePkgFull extends MemePkg {
  pkg_name: string,
  pkg_author?: string,
  pkg_upstream?: string,
}

export function collectTag(tags: Tag[]): TagM[] {
  const tagM = R.reduce((pre: Map<string, TagM> | undefined, cur: Tag) => {
    if (!pre!.has(cur.key)) {
      pre!.set(cur.key, {
        key: cur.key,
        value: [cur.value]
      })
    } else {
      pre!.get(cur.key)?.value.push(cur.value)
    }
    return pre
  }, new Map<string, TagM>(), tags)!

  return R.map(R.last, [...tagM]) as TagM[]
}