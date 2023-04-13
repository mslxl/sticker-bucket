
<script setup lang="ts">
const props = withDefaults(defineProps<{vertical?: boolean, dark?: boolean}>(), {
  vertical: false,
  dark: false
})
</script>

<template lang="pug">
.divider(:class="{vertical: props.vertical, dark:props.dark}")
</template>

<style scoped lang="scss">


.divider {
  --divider-color: var(--primary-border-color)

  width: 100%;
  height: 2px;
  border-bottom: 2px solid var(--divider-color);
}

.dark{
  --divider-color: var(--secondary-border-color) !important;
}

.vertical{
  width: 2px !important;
  height: 100% !important;
  border-right: 2px solid var(--divider-color);
  border-bottom: 0px !important;
}


</style>