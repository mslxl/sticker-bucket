import * as R from 'ramda'
import { Autocomplete, ListItem, ListItemText, TextField } from '@mui/material'
import { useMemo } from 'react'
import DividerItem from './DividerItem'

type Option = [string, string]
export interface ComboBoxItemProps {
  title: string,
  secondary?: string

  option: Option[]
  defaultValue: string,
  applyOption?: (option: string) => void,
  applyIndex?: (index: number) => void,
}

export default function ComboBoxItem(props: ComboBoxItemProps) {

  const optionDisplay = useMemo(()=>{
    return R.map(R.head, props.option) as string[]
  }, [props.option])

  const defaultDisplay = useMemo(()=>{
    const defaultValueIndex = R.map(R.last, props.option).indexOf(props.defaultValue)
    return props.option[defaultValueIndex][0]
  }, [props.defaultValue, optionDisplay])

  function emitChange(value: string) {
    const idx = optionDisplay.indexOf(value)
    if (props.applyOption) {
      props.applyOption(props.option[idx][1])
    }
    if (props.applyIndex) {
      props.applyIndex(idx)
    }
  }

  return (
    <>
      <ListItem
        key={props.title}
        disablePadding
        secondaryAction={
          <Autocomplete
            // disablePortal
            options={optionDisplay}
            value={defaultDisplay}
            disableClearable
            onChange={(_, v) => emitChange(v)}
            sx={{ width: 150 }}
            renderInput={(params) => <TextField {...params} variant="outlined" />}
          />
        }>
        <ListItemText primary={props.title} secondary={props.secondary} />
      </ListItem>
      <DividerItem />
    </>
  )

}