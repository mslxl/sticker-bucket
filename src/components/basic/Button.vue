<script setup lang="ts">
const props = defineProps<{
  justify?: 'left' | 'center'| 'right'
}>()
</script>
<template lang="pug">
.mmm-button(:class="'btn-' + (props.justify? props.justify : 'center')")
  .btn-content
    slot
</template>
<style scoped lang="scss">
.mmm-button {
  padding: 8px;
  transition: background-color .3s;
  border-radius: 5px;
  min-width: 48px;
  .btn-content{
    width: 100%;
    height: 100%;
  }
}

.btn-left{
  align-items: start;
  text-align: left;
}
.btn-center{
  text-align: center;
  align-items: center;
}

.btn-right{
  align-items: end;
  text-align: right;
}

.mmm-button:hover{
  background-color: var(--hover-color);
}
</style>