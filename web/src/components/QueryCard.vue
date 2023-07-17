<template>
  <Card>
    <PVPQueryCardBody :modes="query.modes" :rules="query.rules" :includes="query.includes" v-if="qtype == 'pvp'" />
    <div class="p-4 py-2 sm:py-3 lg:py-4">
      <span class="text-2xl font-bold tracking-tight fmt-text-primary pr-2">{{ qtype.toUpperCase() }}</span>
      <span class="font-normal fmt-text-secondary">{{ localTime }}</span>
    </div>
  </Card>
</template>

<script setup>
import { computed } from 'vue';
import PVPQueryCardBody from './PVPQueryCardBody.vue';
import Card from './Card.vue';

const props = defineProps({
  qtype: String,
  query: Object,
  createdTime: String,
})

const localTime = computed(() => new Date(props.createdTime + " UTC").toLocaleDateString());
</script>