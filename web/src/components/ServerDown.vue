<template>
  <div class="
    select-none
    px-3 py-1 
    leading-none 
    rounded-full 
    bg-red-200 dark:bg-red-900 
    text-lg font-medium text-center 
    text-red-800 dark:text-red-200">
    server failed to response.
  </div>
</template>

<script setup>
import { onMounted } from 'vue';
import { initFlowbite } from 'flowbite'

onMounted(initFlowbite);
</script>
