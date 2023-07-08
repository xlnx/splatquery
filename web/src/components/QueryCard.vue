<template>
  <Card>
    <PVPQueryCardBody :createdTime="createdTime" :modes="query.modes" :rules="query.rules" :includes="query.includes"
      v-if="qtype == 'pvp'" />
  </Card>
</template>

<script setup>
import PVPQueryCardBody from './PVPQueryCardBody.vue';
import Card from './Card.vue';

defineProps({
  qtype: String,
  query: Object,
  createdTime: String,
})

</script>