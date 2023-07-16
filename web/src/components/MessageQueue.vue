<template>
  <div class="absolute top-[88px] right-[16px] z-50">
    <div class="relative">
      <TransitionGroup name="mq-li" tag="ul" class="space-y-4">
        <div v-for="e in toasts.slice().reverse()" :key="e.id" class="flex justify-end">
          <button role="alert" @click="dismiss(e.id)" class="
          w-fit max-w-xs sm:max-w-sm
          flex items-center 
          p-4 space-x-4 space-x 
          select-none transition-all
          rounded-lg shadow-md
          border-1 border-gray-300 dark:border-gray-600 
          divide-x divide-gray-200 dark:divide-gray-700 
          bg-white dark:bg-gray-800">
            <div v-if="e.level == 'success'" class="text-green-500">
              <svg class="w-6 h-6" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                <path
                  d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 8.207-4 4a1 1 0 0 1-1.414 0l-2-2a1 1 0 0 1 1.414-1.414L9 10.586l3.293-3.293a1 1 0 0 1 1.414 1.414Z" />
              </svg>
            </div>
            <div v-if="e.level == 'error'" class="text-red-500">
              <svg class="w-6 h-6" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                <path
                  d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 11.793a1 1 0 1 1-1.414 1.414L10 11.414l-2.293 2.293a1 1 0 0 1-1.414-1.414L8.586 10 6.293 7.707a1 1 0 0 1 1.414-1.414L10 8.586l2.293-2.293a1 1 0 0 1 1.414 1.414L11.414 10l2.293 2.293Z" />
              </svg>
            </div>
            <div v-if="e.level == 'info'" class="text-gray-500 dark:text-gray-400">
              <svg class="w-6 h-6" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                <path
                  d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 8.207-4 4a1 1 0 0 1-1.414 0l-2-2a1 1 0 0 1 1.414-1.414L9 10.586l3.293-3.293a1 1 0 0 1 1.414 1.414Z" />
              </svg>
            </div>
            <div v-if="e.level == 'warning'" class="bg-orange-500 dark:bg-orange-700">
              <svg class="w-6 h-6" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor"
                viewBox="0 0 20 20">
                <path
                  d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5ZM10 15a1 1 0 1 1 0-2 1 1 0 0 1 0 2Zm1-4a1 1 0 0 1-2 0V6a1 1 0 0 1 2 0v5Z" />
              </svg>
            </div>
            <div class="flex-1 pl-4 fmt-sm font-normal text-left text-gray-500 dark:text-gray-400">
              {{ e.message }}
            </div>
          </button>
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';

const timeOutSec = 8;

const toasts = ref([]);

const success = msg => prompt('success', msg);
const info = msg => prompt('info', msg);
const warning = msg => prompt('warning', msg);
const error = msg => prompt('error', msg);

const prompt = (level, message) => {
  const id = Math.floor(Math.random() * 10001221).toString(16);
  toasts.value.push({ id, level, message });
  setTimeout(() => dismiss(id), timeOutSec * 1000);
  // developer panel
  if (level == 'error') {
    console.error(message);
  }
}

const dismiss = id => {
  const idx = toasts.value.findIndex(e => e.id == id);
  if (idx >= 0) {
    toasts.value.splice(idx, 1);
  }
}

defineExpose({
  success,
  info,
  warning,
  error,
})
</script>

<style>
.mq-li-enter-active,
.mq-li-leave-active {
  transition: all 0.5s ease;
}

.mq-li-enter-from,
.mq-li-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>
