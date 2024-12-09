import { useInfiniteQuery, useQuery } from '@tanstack/vue-query'
import { last } from 'lodash/fp'
import { commands } from '@/lib/client'

export function useCollectionsQuery(name: string) {
  return useQuery({
    queryKey: ['collections', name],
    queryFn: () => commands.searchCollections(name),
  })
}

export function useCollectionsInfiniteQuery(order: 'ASC' | 'DESC' = 'DESC', limit: number = 20) {
  return useInfiniteQuery({
    queryKey: ['collections', 'infinite', order],
    initialPageParam: null as string | null,
    queryFn: async ({ pageParam }) => commands.getCollections(order, limit, pageParam),
    getNextPageParam: lastGroup => last(lastGroup)?.id,
  })
}
