<template>
  <div class="relative overflow-hidden aspect-[4/1]">
    <img class="opacity-75 w-full" :src="getPVPStageImgUrl(includes[0])" />
    <div class="
        absolute top-0 left-0 w-full h-full 
        flex justify-end
        p-4 backdrop-blur-sm">
      <img v-for="mode in modes" :src="getModeImgUrl(mode)" />
      <img v-for="rule in rules" :src="getRuleImgUrl(rule)" />
    </div>
  </div>
  <div class="p-4">
    <span class="text-2xl font-bold tracking-tight text-gray-900 dark:text-white pr-2">PVP</span>
    <span class="font-normal text-gray-700 dark:text-gray-400">{{ localTime }}</span>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { getModeImgUrl, getRuleImgUrl, getPVPStageImgUrl } from '../utils'

const props = defineProps({
  modes: Array,
  rules: Array,
  includes: Array,
  createdTime: String,
})

const localTime = computed(() => new Date(props.createdTime + " UTC").toLocaleDateString());

</script>