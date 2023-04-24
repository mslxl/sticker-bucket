import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useSettings = defineStore('settings', ()=>{
  const theme = ref('dark')
  const colorschema = ref('blue')
  const debug = ref(false)
  

  return {
    theme,
    colorschema,
    debug
  }
})
