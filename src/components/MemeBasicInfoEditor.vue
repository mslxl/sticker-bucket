<script setup lang="ts">

import { library } from '@fortawesome/fontawesome-svg-core'
import { faTurnDown } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import { computed, watch, ref } from 'vue'
import { queryNamespaceWithPrefix, queryTagValueWithPrefix } from '../scripts/rs/db'

import { ElCheckbox, ElInput, ElButton, ElAutocomplete } from 'element-plus'
import MTag from '../components/basic/Tag.vue'


library.add(faTurnDown)
interface Tag {
  namespace: string,
  value: string,
}

const props = defineProps<{
  deleteFile: boolean,
  summary: string,
  description: string
  tags: Tag[]
}>()

const emits = defineEmits<{
  (e: 'update:deleteFile', value: boolean): void
  (e: 'update:summary', value: string): void
  (e: 'update:description', value: string): void
  (e: 'update:tags', value: Tag[]): void
}>()

const imageTagsKV = computed(() => props.tags.reduce((map: Map<string, string[]>, cur: Tag) => {
  if (map.has(cur.namespace)) {
    map.get(cur.namespace)!.push(cur.value)
  } else {
    map.set(cur.namespace, [cur.value])
  }
  return map
}, new Map<string, string[]>()))

const imageNamespace = computed(() => Array.from(imageTagsKV.value.keys()))
watch(imageTagsKV, (v) => {
  console.log(v)
})

const tagInput = ref('')

const autoComplateField = ref<InstanceType<typeof ElAutocomplete>>()

function triggerValueComplate(){
  if(tagInput.value.endsWith(':')){
    autoComplateField.value!.blur()
    setTimeout(()=>{
      autoComplateField.value!.focus()
      autoComplateField.value!.inputRef!.input!.setSelectionRange(tagInput.value.length, tagInput.value.length)
    }, 100)
  }
}

function fetchTagComplate(queryString: string, callback: (arg: any) => void) {
  let coloIdx = queryString.indexOf(':')
  if (coloIdx != -1) {
    let namespace = queryString.substring(0, coloIdx)
    queryTagValueWithPrefix(namespace, queryString.substring(coloIdx + 1)).then((value) => {
      callback(value.sort().map(v => { return { value: `${namespace}:${v}`, display: v, type: 'Value' } }))
    })
  } else {
    queryNamespaceWithPrefix(queryString).then((value)=>{
      callback(value.sort().map(v => { return { value: `${v}:`, display: v, type: 'Namespace'}}))
    })
  }
}


function addNewTag() {
  const legalRegex = /(.+):(.+)/
  const result = legalRegex.exec(tagInput.value)
  if (!result) return
  const namespace = result[1].trim()
  const value = result[2].trim()

  let newTags = new Set<Tag>(props.tags)
  newTags.add({
    namespace,
    value
  })
  emits('update:tags', Array.from(newTags.values()))
  tagInput.value = ''
}

function removeTag(namespace: string, value: string) {
  let newTags = props.tags.filter((tag) => !(tag.namespace == namespace && tag.value == value))
  emits('update:tags', newTags)
}

</script>
<template lang="pug">
.editor
  div
  el-checkbox(
    label="Delete image file"
    :model-value="deleteFile"
    @update:model-value="value => emits('update:deleteFile', Boolean(value))"
  )

  span Summary
  el-input(
    :model-value="summary"
    @update:model-value="value => emits('update:summary', value)")

  span Description
  el-input(
    :model-value="description"
    @update:model-value="value=> emits('update:description', value)")

  span Tag
  .autocomplete__wrapper
    el-autocomplete(
      ref="autoComplateField"
      style="flex-grow: 1;"
      v-model="tagInput"
      :fetch-suggestions="fetchTagComplate"
      @select="triggerValueComplate"
      clearable
    )
      template(#default="{ item }")
        .complate-item
          span.display {{ item.display }}
          .flex-space
          span.type {{ item.type }}
    el-button(
      text
      type=""
      @click="addNewTag")
      font-awesome-icon(icon="fa-solid fa-turn-down fa-rotate-90")
  template(v-for="nsp in imageNamespace" :key="nsp")
    m-tag {{  nsp }}
    .tag-value
      m-tag.tag-item(
        v-for="value in imageTagsKV.get(nsp)"
        :key="nsp+value"
        :closable="true"
        @on-close="removeTag(nsp, value)") {{  value  }}

</template>

<style scoped lang="scss">
.editor {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 8px;
}
.autocomplete__wrapper{
  display: flex;
  align-items: center;
}
.complate-item{
  display: flex;
  .flex-space{
    flex-grow: 1;
  }
  .type{
    font-size: smaller;
    font-style: italic;
  }
}
</style>