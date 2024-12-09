import antfu from '@antfu/eslint-config'
import importPlugin from 'eslint-plugin-import'

export default antfu(
  {
    vue: true,
  },
  importPlugin.flatConfigs.recommended,
  {
    rules: {
      'vue/multi-word-component-names': 'off',
      'import/no-relative-parent-imports': 'error',
      'import/no-unresolved': 'off',
      'import/named': 'off',
    },
  },
)
