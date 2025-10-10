import type { Prefs } from '@/lib/client'
import { useMutation, useQuery } from '@tanstack/vue-query'
import { produce } from 'immer'
import { commands } from '@/lib/client'

export function usePrefs() {
  return useQuery({
    queryKey: ['prefs'],
    queryFn: () => commands.getPrefs(),
  })
}

export function usePrefsMutate() {
  const data = usePrefs()

  const mutation = useMutation({
    mutationKey: ['prefs'],
    mutationFn: (prefs: Prefs) => commands.setPrefs(prefs),
  })

  return (recipe: (prefs: Prefs) => void | Prefs) => data.promise.value.then((prefs) => {
    return mutation.mutateAsync(produce(prefs, recipe))
  })
}
