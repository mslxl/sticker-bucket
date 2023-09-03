import * as R from 'ramda'
import { Chip, Stack } from '@mui/material'
import { Tag, TagM } from '../model/meme'
import { Lock as LockIcon } from '@mui/icons-material'

export interface TagShowcaseProps {
  tags: TagM[],
  lockTag?: Tag[],
  lockable?: boolean,
  handleLock?: (key: string, value: string) => void,
  handleDelete?: (key: string, value: string) => void,
}
export default function TagShowcase({ tags, lockTag, lockable, handleLock, handleDelete }: TagShowcaseProps) {
  return (
    <Stack spacing={1} direction="column">
      {
        tags.map(namespace => (
          <Stack key={namespace.key} spacing={1} direction="row">
            <Chip label={namespace.key} variant='outlined' />
            {
              namespace.value.map((value) => ({
                lock: lockTag ? R.includes({ key: namespace.key, value }, lockTag) : false,
                tag: { key: namespace.key, value },
              })).map(({ lock, tag }) =>
                <Chip
                  key={tag.value}
                  label={tag.value}
                  avatar={lock ? <LockIcon /> : undefined}
                  clickable={lockable}
                  onClick={() => lockable && handleLock && handleLock(tag.key, tag.value)}
                  onDelete={(lock || !handleDelete) ? undefined : (() => handleDelete(tag.key, tag.value))} />
              )
            }
          </Stack>
        ))
      }

    </Stack>
  )


}