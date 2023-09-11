import * as R from 'ramda'
import { test, expect } from 'vitest'
import enUS from './en_US'
import zhCN from './zh_CN'

const lngSet = [enUS, zhCN]
test('I18N integrity validate', () => {
  const keys = R.uniq(R.reduce(R.concat,[], R.map(Object.keys, lngSet))) as string[]

  for(const lng of R.map(Object.keys,lngSet)){
    expect(lng).toEqual(keys)
  }
})

