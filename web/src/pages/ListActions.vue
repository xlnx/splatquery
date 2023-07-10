<template>
  <div class="grid grid-flow-row-dense grid-cols-1 lg:grid-cols-2" v-if="!!response">
    <div class="m-2 sm:m-3 lg:m-4">
      <WebPushActionCard :defaultActive="active.webpush" :defaultConfig="response.webpush" />
    </div>
  </div>

  <div v-if="!response && !failed">
    <Loading />
  </div>

  <div v-if="failed">
    <ServerDown />
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { initFlowbite } from 'flowbite'
import axios from 'axios';
import { backOff } from "exponential-backoff";
import Loading from '../components/Loading.vue';
import ServerDown from '../components/ServerDown.vue';
import WebPushActionCard from '../components/WebPushActionCard.vue';

onMounted(initFlowbite);

const response = ref();
const active = ref({});
const failed = ref();

onMounted(async () => {
  try {
    response.value = await backOff(async () => {
      const response = await axios.get(import.meta.env.VITE_API_SERVER + '/action/list');
      if (response.status != 200) {
        throw response;
      }
      return response.data;
    }, {
      numOfAttempts: 5,
    });
    for (let action of response.value.actions) {
      active.value[action] = true;
    }
  } catch (err) {
    console.error(err);
    failed.value = true;
  }
})
</script>