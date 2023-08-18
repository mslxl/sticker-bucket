import * as R from 'ramda'
export interface Tag{
  key: string,
  value: string,
}
export interface TagM{
  key: string,
  value: string[]
}
export interface Meme{
  id: string | null,
  name:string,
  description: string,
  tags: Tag[],
}

export function collectTag(tags: Tag[]): TagM[]{
  const tagM =  R.reduce((pre:Map<string, TagM>|undefined, cur: Tag)=>{
    if(!pre!.has(cur.key)){
      pre!.set(cur.key, {
        key: cur.key,
        value: [cur.value]
      })
    }else{
      pre!.get(cur.key)?.value.push(cur.value)
    }
    return pre
  }, new Map<string, TagM>(), tags)!

  return R.map(R.last, [...tagM]) as TagM[]
}