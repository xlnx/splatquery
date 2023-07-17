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
</template>

<script setup>
import { getModeImgUrl, getRuleImgUrl, getPVPStageImgUrl } from '../utils'

const props = defineProps({
  modes: Array,
  rules: Array,
  includes: Array,
})
</script>