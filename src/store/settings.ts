import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useSettings = defineStore('settings', ()=>{
  const theme = ref('dark')

  

  return {
    theme
  }
})
