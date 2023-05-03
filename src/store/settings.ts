import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useSettings = defineStore('settings', ()=>{
  const debug = ref(false)
  

  return {
    debug
  }
})
