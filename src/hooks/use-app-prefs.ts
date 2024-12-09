import type { AppPrefs } from '@/lib/client'
import { useMutation, useQuery } from '@tanstack/vue-query'
import { produce } from 'immer'
import { commands } from '@/lib/client'

export function useAppPrefs() {
  return useQuery({
    queryKey: ['app-prefs'],
    queryFn: () => commands.getAppPrefs(),
  })
}

export function useAppPrefsMutate() {
  const data = useAppPrefs()

  const mutation = useMutation({
    mutationKey: ['app-prefs'],
    mutationFn: (prefs: AppPrefs) => commands.setAppPrefs(prefs),
  })

  return (recipe: (prefs: AppPrefs) => void | AppPrefs) => data.promise.value.then((prefs) => {
    return mutation.mutateAsync(produce(prefs, recipe))
  })
}
