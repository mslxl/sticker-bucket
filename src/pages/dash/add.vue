<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import * as dialog from '@tauri-apps/plugin-dialog'
import { readTextFile } from '@tauri-apps/plugin-fs'
import * as log from '@tauri-apps/plugin-log'
import { toTypedSchema } from '@vee-validate/zod'
import { debounce } from 'lodash/fp'
import { Check, Plus, Search } from 'lucide-vue-next'
import { AnimatePresence, motion } from 'motion-v'
import { match, P } from 'ts-pattern'
import { useForm } from 'vee-validate'
import { computed, nextTick, ref } from 'vue'
import { toast } from 'vue-sonner'
import * as z from 'zod'
import CollectionCreate from '@/components/CollectionCreate.vue'
import Markdown from '@/components/Markdown.vue'
import Image from '@/components/media/Image.vue'
import Video from '@/components/media/Video.vue'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxItemIndicator, ComboboxList } from '@/components/ui/combobox'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { TagsInput, TagsInputInput, TagsInputItem, TagsInputItemDelete, TagsInputItemText } from '@/components/ui/tags-input'
import { Textarea } from '@/components/ui/textarea'
import { usePrefs } from '@/hooks/use-prefs'
import { commands } from '@/lib/client'
import { cn } from '@/lib/utils'

const props = withDefaults(defineProps<{
  class?: string
}>(), {
  class: '',
})

const formSchema = toTypedSchema(z.object({
  name: z.string().optional(),
  description: z.string().optional(),
  collection: z.object({
    label: z.string(),
    value: z.object({
      typ: z.enum(['use', 'new']),
      id: z.string(),
    }),
  }),
  tags: z.array(z.string()).optional(),
  delete_assets: z.boolean().default(false),
}))

interface Asset {
  ref: string
  type: 'image' | 'video' | 'text'
}

const assetsToAppend = ref<Asset[]>([])
const selectedAssetIndex = ref<number | null>(null)
const selectedAsset = computed(() => {
  return selectedAssetIndex.value !== null ? assetsToAppend.value[selectedAssetIndex.value] : null
})

const collectionCompletition = ref<{ label: string, value: { typ: 'use' | 'new', id: string } }[]>([
  {
    label: 'Inbox',
    value: {
      typ: 'use',
      id: 'internal.inbox',
    },
  },
])

const isCreatingCollectionDialogVisible = ref(false)
const creatingCollectionDefaultName = ref('')

const form = useForm({
  validationSchema: formSchema,
  initialValues: {
    name: '',
    description: '',
    collection: {
      label: 'Inbox',
      value: {
        typ: 'use',
        id: 'internal.inbox',
      },
    },
    tags: [],
    delete_assets: false,
  },
})

const prefs = usePrefs()

prefs.promise.value.then((prefs) => {
  form.setFieldValue('delete_assets', prefs.delete_assets_on_add)
})

const refreshCollectionCompletition = debounce(500, async (input: string) => {
  collectionCompletition.value = []
  const completitions: typeof collectionCompletition.value = []

  if (input.trim().length !== 0) {
    completitions.push({
      label: `Create new collection: ${input}`,
      value: {
        typ: 'new',
        id: input,
      },
    })
  }
  const collections = await commands.searchCollections(input)
  completitions.push(...collections.map(collection => ({
    label: collection.name,
    value: {
      typ: 'use' as const,
      id: collection.id,
    },
  })))

  log.trace(`Completitions: ${JSON.stringify(completitions)}`)
  collectionCompletition.value = completitions
})

function handleCreateCollection(name: string) {
  creatingCollectionDefaultName.value = name
  nextTick(() => {
    isCreatingCollectionDialogVisible.value = true
  })
}
function handleCreateCollectionSuccess(collectionId: string) {
  form.setFieldValue('collection', {
    label: creatingCollectionDefaultName.value,
    value: {
      typ: 'use',
      id: collectionId,
    },
  })
  toast.success(`Collection ${creatingCollectionDefaultName.value} created`)
  isCreatingCollectionDialogVisible.value = false
  creatingCollectionDefaultName.value = ''
}
function handleCreateCollectionCancel() {
  form.setFieldValue('collection', {
    label: 'Inbox',
    value: {
      typ: 'use',
      id: 'internal.inbox',
    },
  })
  toast.info(`Collection creation cancelled`)
  isCreatingCollectionDialogVisible.value = false
  creatingCollectionDefaultName.value = ''
}

function displayCollectionCompletition(val: typeof collectionCompletition.value[number]) {
  return match(val)
    .with({ value: { typ: 'use' } }, v => v.label)
    .with({ value: { typ: 'new' } }, v => v.value.id)
    .exhaustive()
}

const handleSubmit = form.handleSubmit(async (values) => {
  // toast(
  //   markRaw(() => h('div', {
  //     class: 'mt-2 w-[340px] rounded-md bg-slate-950 p-4',
  //   }, [
  //     h('pre', {
  //       class: 'text-white',
  //     }, JSON.stringify(values, null, 2)),
  //   ]),
  //   ),
  // )
  try {
    if (values.name?.trim().length === 0 && assetsToAppend.value.length === 0) {
      throw new Error('New sticker cannot be empty')
    }
    await commands.addSticker(
      assetsToAppend.value.map(v =>
        match(v)
          .with({ type: 'image' }, v => ({
            type: 'image' as const,
            c: v.ref,
          }))
          .with({ type: 'video' }, v => ({
            type: 'video' as const,
            c: v.ref,
          }))
          .with({ type: 'text' }, v => ({
            type: 'text' as const,
            c: v.ref,
          }))
          .exhaustive(),
      ),
      values.name ?? null,
      values.description ?? null,
      values.collection.value.id,
      values.tags ?? [],
      values.delete_assets,
    )
    toast.success(`Sticker added`)

    // reset form
    form.resetForm()
    form.setFieldValue('delete_assets', values.delete_assets)
    assetsToAppend.value = []
    selectedAssetIndex.value = null
  }
  catch (e) {
    toast.error(`Failed to add sticker: ${e}`)
  }
})

async function handleAddFile() {
  try {
    const files = await dialog.open({
      multiple: true,
      filters: [
        { name: 'Media', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'avif', 'jfif', 'mp4', 'webm', 'txt', 'md', 'markdown'] },
        { name: 'Image', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'avif', 'jfif'] },
        { name: 'Video', extensions: ['mp4', 'webm'] },
        { name: 'Text', extensions: ['txt'] },
      ],
    })
    if (!files)
      return
    const appended = await Promise.all(files.map(file =>
      match(file.toLowerCase())
        .with(P.string.regex(/.*\.(png|jpg|jpeg|gif|webp|avif|jfif)$/), () => ({
          ref: file,
          type: 'image' as const,
        }))
        .with(P.string.regex(/.*\.(mp4|webm)$/), () => ({
          ref: file,
          type: 'video' as const,
        }))
        .otherwise(async () => ({
          type: 'text' as const,
          ref: await readTextFile(file),
        })),
    ))
    assetsToAppend.value = [...assetsToAppend.value, ...appended]
    if (selectedAssetIndex.value === null) {
      selectedAssetIndex.value = 0
    }
  }
  catch (e) {
    await dialog.message(`Failed to add file: ${e}`, {
      title: 'Error',
      kind: 'error',
    })
  }
}
async function handleClean() {
  const result = await dialog.confirm('Are you sure you want to clean the assets?')
  if (result) {
    assetsToAppend.value = []
    selectedAssetIndex.value = null
  }
}
</script>

<template>
  <ScrollArea :class="cn('min-h-[calc(100vh-var(--spacing)*16)] h-full', props.class)">
    <div class="flex flex-row items-stretch px-4 min-h-[calc(1/3*100vh)] max-h-[calc(2/3*100vh)] transition">
      <div class="flex-1 overflow-clip flex assets-preview-container" :alt="`Selected Asset: ${selectedAssetIndex}`">
        <AnimatePresence mode="popLayout">
          <motion.div
            v-if="selectedAsset === null"
            :initial="{ opacity: 0 }"
            :animate="{ opacity: 1 }"
            :exit="{ opacity: 0 }"
            :transition="{ duration: 0.5 }"
          >
            <div class="h-[calc(1/3*100vh)] flex items-center justify-center mb-2">
              <div class="size-full bg-black/50 rounded-md" />
            </div>
          </motion.div>
          <motion.div
            v-else-if="selectedAsset.type === 'image'"
            :key="`image-${selectedAsset.ref}`"
            :initial="{ opacity: 0 }"
            :animate="{ opacity: 1 }"
            :exit="{ opacity: 0 }"
          >
            <Image :src="convertFileSrc(selectedAsset.ref)" />
          </motion.div>
          <motion.div
            v-else-if="selectedAsset.type === 'video'"
            :key="`video-${selectedAsset.ref}`"
            :initial="{ opacity: 0 }"
            :animate="{ opacity: 1 }"
            :exit="{ opacity: 0 }"
          >
            <Video :src="convertFileSrc(selectedAsset.ref)" />
          </motion.div>
          <motion.div
            v-else-if="selectedAsset.type === 'text'"
            :key="`text-${selectedAsset.ref}`"
            :initial="{ opacity: 0 }"
            :animate="{ opacity: 1 }"
            :exit="{ opacity: 0 }"
          >
            <Markdown :content="selectedAsset.ref" />
          </motion.div>
        </AnimatePresence>
      </div>
      <ScrollArea class="border">
        <ul class="divide-y">
          <li
            v-for="(asset, index) in assetsToAppend"
            :key="asset.ref"
            class="cursor-pointer h-8"
            @click="selectedAssetIndex = index"
          >
            <div v-if="asset.type === 'image'" class="h-full w-full">
              <img class="h-full object-cover" :src="convertFileSrc(asset.ref)" alt="Image">
            </div>
            <div v-if="asset.type === 'video'" class="h-full w-full">
              <video class="h-full object-cover" :src="convertFileSrc(asset.ref)" alt="Video" muted loop autoplay />
            </div>
            <p v-if="asset.type === 'text'">
              {{ asset.ref }}
            </p>
          </li>
        </ul>
      </ScrollArea>
    </div>
    <div class="space-x-1 px-4 justify-end flex">
      <Button @click="handleAddFile">
        Add File
      </Button>
      <Button>Screenshot</Button>
      <Button>Clipboard</Button>
      <Button variant="destructive" :disabled="assetsToAppend.length === 0" @click="handleClean">
        Clean
      </Button>
    </div>
    <form class="space-y-2 m-2 p-4 border" @submit="handleSubmit">
      <FormField v-slot="{ componentField }" name="collection">
        <FormItem>
          <FormLabel>Collection</FormLabel>
          <Combobox by="label" :model-value="componentField.modelValue" @update:model-value="componentField['onUpdate:modelValue']">
            <FormControl>
              <ComboboxAnchor>
                <div class="relative w-full max-w-sm items-center">
                  <ComboboxInput class="pl-9" autocomplete="off" :display-value="displayCollectionCompletition" @update:model-value="refreshCollectionCompletition" />
                  <span class="absolute start-0 inset-y-0 flex items-center justify-center px-3">
                    <Search class="size-4 text-muted-foreground" />
                  </span>
                </div>
              </ComboboxAnchor>
            </FormControl>
            <ComboboxList>
              <ComboboxEmpty>
                No collections found
              </ComboboxEmpty>
              <ComboboxGroup>
                <template v-for="collection in collectionCompletition" :key="collection.value.id">
                  <ComboboxItem v-if="collection.value.typ === 'use'" :value="collection">
                    {{ collection.label }}
                    <ComboboxItemIndicator>
                      <Check class="ml-auto size-4" />
                    </ComboboxItemIndicator>
                  </ComboboxItem>
                  <ComboboxItem v-else-if="collection.value.typ === 'new'" :value="collection" @click="handleCreateCollection(collection.value.id)">
                    {{ collection.label }}
                    <Plus class="ml-auto size-4" />
                  </ComboboxItem>
                </template>
              </ComboboxGroup>
            </ComboboxList>
          </Combobox>
          <FormMessage />
        </FormItem>
      </FormField>
      <FormField v-slot="{ componentField }" name="name">
        <FormItem>
          <FormLabel>Name</FormLabel>
          <FormControl>
            <Input type="text" v-bind="componentField" autocomplete="off" />
          </FormControl>
          <FormMessage />
        </FormItem>
      </FormField>
      <FormField v-slot="{ componentField }" name="description">
        <FormItem>
          <FormLabel>Description</FormLabel>
          <FormControl>
            <Textarea v-bind="componentField" autocomplete="off" />
          </FormControl>
          <FormMessage />
        </FormItem>
      </FormField>
      <FormField v-slot="{ componentField }" name="tags">
        <FormItem>
          <FormLabel>Tags</FormLabel>
          <FormControl>
            <TagsInput
              :model-value="componentField.modelValue"
              @update:model-value="componentField['onUpdate:modelValue']"
            >
              <TagsInputItem v-for="item in componentField.modelValue" :key="item" :value="item">
                <TagsInputItemText />
                <TagsInputItemDelete />
              </TagsInputItem>
              <TagsInputInput placeholder="Tags..." />
            </TagsInput>
          </FormControl>
          <FormMessage />
        </FormItem>
      </FormField>
      <FormField v-slot="{ value, handleChange }" type="checkbox" name="delete_assets">
        <FormItem class="flex flex-row items-start gap-x-3 space-y-0 rounded-md border p-4">
          <FormControl>
            <Checkbox :model-value="value" @update:model-value="handleChange" />
          </FormControl>
          <div class="space-y-1 leading-none">
            <FormLabel>Delete original assets</FormLabel>
            <FormDescription>
              If enabled, the original assets will be deleted after adding the stickers.
            </FormDescription>
            <FormMessage />
          </div>
        </FormItem>
      </FormField>
      <Button type="submit">
        Add
      </Button>
    </form>
    <Dialog v-model:open="isCreatingCollectionDialogVisible">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create new collection: {{ creatingCollectionDefaultName }}</DialogTitle>
        </DialogHeader>
        <CollectionCreate
          :default-name="creatingCollectionDefaultName"
          @cancel="handleCreateCollectionCancel"
          @create="handleCreateCollectionSuccess"
        />
      </DialogContent>
    </Dialog>
  </ScrollArea>
</template>

<style lang="scss" scoped>
.assets-preview-container{
    & > * {
        width: 100%;
        height: 100%;
    }
}
</style>
