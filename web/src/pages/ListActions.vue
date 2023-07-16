<template>
  <div class="grid grid-flow-row-dense grid-cols-1 lg:grid-cols-2" v-if="!!actions">
    <div class="m-2 sm:m-3 lg:m-4">
      <WebPushActionCard :defaultActive="actions.webpush.active" :defaultConfig="actions.webpush.sub" />
    </div>
  </div>

  <div v-if="!actions && !failed">
    <Loading />
  </div>

  <div v-if="failed">
    <ServerDown />
  </div>
</template>

<script setup>
import { ref, onMounted, inject } from 'vue';
import { initFlowbite } from 'flowbite'
import axios from 'axios';
import { backOff } from "exponential-backoff";
import Loading from '../components/Loading.vue';
import ServerDown from '../components/ServerDown.vue';
import WebPushActionCard from '../components/WebPushActionCard.vue';

onMounted(initFlowbite);

const mq = inject('mq');
const actions = ref();
const failed = ref();

onMounted(async () => {
  try {
    const li = await backOff(async () => {
      const response = await axios.get(import.meta.env.VITE_API_SERVER + '/action/list');
      if (response.status != 200) {
        throw response;
      }
      return response.data;
    }, {
      numOfAttempts: 5,
    });
    const map = {};
    for (let { id, agent, active, ext_info } of li) {
      if (!(agent in map)) {
        map[agent] = { active, sub: [] };
      }
      map[agent].sub.push({ id, ...ext_info });
    }
    actions.value = {
      webpush: { sub: [] },
      ...map,
    }
  } catch (err) {
    mq.value.error(err);
    failed.value = true;
  }
})
</script>