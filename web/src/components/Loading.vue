<template>
  <div class="
    select-none
    px-3 py-1 
    leading-none 
    rounded-full 
    animate-pulse 
    bg-blue-200 dark:bg-blue-900 
    text-lg font-medium text-center 
    text-blue-800 dark:text-blue-200">
    waiting for server response...
  </div>
</template>

<script setup>
import { onMounted } from 'vue';
import { initFlowbite } from 'flowbite'

onMounted(initFlowbite);
</script>
