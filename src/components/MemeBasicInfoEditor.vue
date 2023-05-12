<script setup lang="ts">

import { library } from '@fortawesome/fontawesome-svg-core'
import { faTurnDown, faLock } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

import { computed, watch, ref } from 'vue'
import { queryNamespaceWithPrefix, queryTagValueWithPrefix } from '../scripts/rs/db'

import { ElCheckbox, ElInput, ElButton, ElAutocomplete } from 'element-plus'
import { ElTag, ElSpace } from 'element-plus'


library.add(faTurnDown, faLock)
interface Tag {
  namespace: string,
  value: string,
  lock?: boolean
}

const props = withDefaults(defineProps<{
  deleteFile: boolean,
  summary: string,
  description: string
  tags: Tag[],
  allowLock?: boolean,
  noDelete?: boolean
}>(), {
  allowLock: false,
  noDelete: false
})

const emits = defineEmits<{
  (e: 'update:deleteFile', value: boolean): void
  (e: 'update:summary', value: string): void
  (e: 'update:description', value: string): void
  (e: 'update:tags', value: Tag[]): void
}>()

const imageTagsKV = computed(() => props.tags.reduce((map: Map<string, Tag[]>, cur: Tag) => {
  if (map.has(cur.namespace)) {
    map.get(cur.namespace)!.push(cur)
  } else {
    map.set(cur.namespace, [cur])
  }
  return map
}, new Map<string, Tag[]>()))

const imageNamespace = computed(() => Array.from(imageTagsKV.value.keys()))

const tagInput = ref('')

const autoComplateField = ref<InstanceType<typeof ElAutocomplete>>()

function clear() {
  emits('update:tags', props.tags.filter((tag) => tag.lock))
  emits('update:description', '')
  emits('update:summary', '')
}


function triggerValueComplate() {
  if (tagInput.value.endsWith(':')) {
    // 重新激活自动补全
    autoComplateField.value!.blur()
    setTimeout(() => {
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
    queryNamespaceWithPrefix(queryString).then((value) => {
      callback(value.sort().map(v => { return { value: `${v}:`, display: v, type: 'Namespace' } }))
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

function toggleLockTag(namespace: string, value: string) {
  if (!props.allowLock) return
  let newTags = props.tags.map(v => {
    if (v.namespace == namespace && v.value == value) {
      if (v.lock) {
        v.lock = false
      } else {
        v.lock = true
      }
    }
    return v
  })
  emits('update:tags', newTags)
}

defineExpose({
  clear,
  triggerValueComplate
})
</script>
<template lang="pug">
.editor
  template(v-if="!noDelete")
    div
    el-checkbox(
      label="Delete image file"
      :model-value="deleteFile"
      @update:model-value="value => emits('update:deleteFile', Boolean(value))"
    )

  span Summary
  el-input(
    :model-value="summary"
    clearable
    @update:model-value="value => emits('update:summary', value)")

  span Description
  el-input(
    :model-value="description"
    clearable
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
    el-tag(size="large" type="info" effect="dark") {{  nsp }}
    .tag-value
      el-tag.tag-item(
        v-for="tag in imageTagsKV.get(nsp)"
        :key="nsp+tag.value"
        type="info"
        size="large"
        closable
        :effect="tag.lock? 'light': 'plain'"
        @click="toggleLockTag(nsp, tag.value)"
        @close="removeTag(nsp, tag.value)")
        el-space
          font-awesome-icon(icon="fa-solid fa-lock" v-if="tag.lock")
          span {{  tag.value  }}

</template>

<style scoped lang="scss">
.editor {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 8px;
}

.autocomplete__wrapper {
  display: flex;
  align-items: center;
}

.complate-item {
  display: flex;

  .flex-space {
    flex-grow: 1;
  }

  .type {
    font-size: smaller;
    font-style: italic;
  }
}</style>