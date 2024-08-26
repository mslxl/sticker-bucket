import {defineConfig} from 'unocss'
import { presetUno } from '@unocss/preset-uno'
import presetAnimations from 'unocss-preset-animations'
import {presetShadcn} from 'unocss-preset-shadcn'

export default defineConfig({
    presets: [
        presetUno(),
        presetAnimations(),
        presetShadcn({
            color: 'blue',
        })
    ]
})