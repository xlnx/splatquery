<template>
  <Card>
    <div data-accordion="collapse" data-active-classes="bg-gray-100 dark:bg-gray-700">
      <button class="p-4 w-full" :data-accordion-target="`#${bodyId}`">
        <div class="flex items-center justify-between font-bold text-3xl fmt-text-primary">
          <span class="flex flex-1">
            <span class="pr-2">{{ name }}</span>
            <slot name="badge" />
          </span>
          <svg data-accordion-icon class="w-3 h-3 rotate-180 shrink-0" xmlns="http://www.w3.org/2000/svg" fill="none"
            viewBox="0 0 10 6">
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M9 5 5 1 1 5" />
          </svg>
        </div>
        <p class="text-left font-normal text-base fmt-text-secondary">{{ brief }}</p>
      </button>
      <div :id="bodyId" class="border-t fmt-border-color fmt-text-primary hidden p-4 fmt-action-card-body">
        <slot />
      </div>
    </div>
  </Card>
</template>

<script setup>
import { computed, onMounted } from 'vue';
import { initFlowbite } from 'flowbite'
import Card from '../components/Card.vue';

onMounted(initFlowbite);

const props = defineProps({
  id: String,
  name: String,
  brief: String,
})

const bodyId = computed(() => `action-card-body-${props.id}`)
</script>