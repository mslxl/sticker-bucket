import { breakpointsTailwind, useBreakpoints } from '@vueuse/core'
import { match } from 'ts-pattern'
import { computed } from 'vue'

export function useGridCols() {
  const breakpoint = useBreakpoints(breakpointsTailwind)
  const bkp = breakpoint.active()
  const cols = computed(() => {
    return match(bkp.value)
      .with('', () => 1)
      .with('sm', () => 2)
      .with('md', () => 3)
      .with('lg', () => 4)
      .with('xl', () => 5)
      .with('2xl', () => 5)
      .exhaustive()
  })
  return { cols, breakpoint: bkp }
}
