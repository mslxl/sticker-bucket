<script setup lang="ts">

</script>
<template lang="pug">
.card
  slot

</template>

<style scoped lang="scss">
.card{
  box-shadow: var(--box-shadow-light);
  border-radius: 4px;
  border: 1px solid var(--primary-border-color);
}
</style>