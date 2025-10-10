<script setup lang="ts">
import { toTypedSchema } from '@vee-validate/zod'
import { useForm } from 'vee-validate'
import { onMounted } from 'vue'
import * as zod from 'zod'
import { commands } from '@/lib/client'
import { Button } from './ui/button'
import { FormControl, FormField, FormLabel, FormMessage } from './ui/form'
import { Input } from './ui/input'
import { Textarea } from './ui/textarea'

const props = withDefaults(defineProps<{
  defaultName?: string
}>(), {
  defaultName: '',
})

const emit = defineEmits<{
  (e: 'cancel'): void
  (e: 'create', values: string): void
}>()

const formSchema = toTypedSchema(zod.object({
  name: zod.string().min(1),
  description: zod.string().optional(),
  author: zod.string().optional(),
}))

const form = useForm({
  validationSchema: formSchema,
  initialValues: {
    name: props.defaultName,
    description: '',
    author: '',
  },
})

onMounted(() => {
  commands.whoami().then((username) => {
    form.setFieldValue('author', username)
  })
})

const handleSubmit = form.handleSubmit(async (values) => {
  const collectionId = await commands.addCollection({
    name: values.name,
    description: values.description ?? null,
    author: values.author ?? null,
  })
  emit('create', collectionId)
})
function handleCancel() {
  form.resetForm()
  emit('cancel')
}
</script>

<template>
  <form class="space-y-4" @submit="handleSubmit">
    <FormField v-slot="{ componentField }" name="name">
      <FormLabel>Name</FormLabel>
      <FormControl>
        <Input type="text" v-bind="componentField" autocomplete="off" />
      </FormControl>
      <FormMessage />
    </FormField>
    <FormField v-slot="{ componentField }" name="author">
      <FormLabel>Author</FormLabel>
      <FormControl>
        <Input type="text" v-bind="componentField" autocomplete="off" />
      </FormControl>
      <FormMessage />
    </FormField>
    <FormField v-slot="{ componentField }" name="description">
      <FormLabel>Description</FormLabel>
      <FormControl>
        <Textarea v-bind="componentField" autocomplete="off" />
      </FormControl>
      <FormMessage />
    </FormField>
    <div class="flex flex-row gap-2">
      <Button type="submit">
        Create
      </Button>
      <Button type="reset" variant="outline" @click="handleCancel">
        Cancel
      </Button>
    </div>
  </form>
</template>
