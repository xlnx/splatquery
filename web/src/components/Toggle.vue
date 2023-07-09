<template>
  <label class="relative inline-flex items-center cursor-pointer">
    <input type="checkbox" value="" class="sr-only peer" :checked="checked" :disabled="disabled"
      @click.prevent="$emit('toggle', $event.target.checked)">
    <div class="
      w-11 h-6 
      bg-gray-300 
      peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 
      dark:peer-focus:ring-blue-800 
      rounded-full 
      peer 
      dark:bg-gray-500 
      peer-checked:after:translate-x-full peer-checked:after:border-white 
      after:content-[''] 
      after:absolute after:top-[2px] after:left-[2px] 
      after:bg-white 
      after:border-gray-300 after:border 
      after:rounded-full 
      after:h-5 after:w-5 after:transition-all 
      dark:border-gray-600 peer-checked:bg-blue-600">
    </div>
  </label>
</template>

<script setup>
import { onMounted } from 'vue';
import { initFlowbite } from 'flowbite'

onMounted(initFlowbite);

defineProps({
  checked: Boolean,
  disabled: {
    type: Boolean,
    default: false,
  }
})
</script>
