<template>
  <div class="grid grid-flow-row-dense grid-cols-1 lg:grid-cols-2" v-if="actions">
    <div class="m-2 sm:m-3 lg:m-4" v-for="agent in agents">
      <ActionCard :id="agent.id" :name="agent.name" :brief="agent.brief" :active="agent.id in actions">
        Not implemented yet.
      </ActionCard>
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
import { ref, onMounted } from 'vue';
import { initFlowbite } from 'flowbite'
import axios from 'axios';
import { backOff } from "exponential-backoff";
import ActionCard from '../components/ActionCard.vue';
import Loading from '../components/Loading.vue';
import ServerDown from '../components/ServerDown.vue';

onMounted(initFlowbite);

const actions = ref();
const failed = ref();

const agents = [{
  id: 'webpush',
  name: 'Web Push',
  brief: 'Receive notifications via your browser.',
},
{
  id: 'ifttt',
  name: 'IFTTT',
  brief: 'Serves as an IFTTT trigger.',
}]

onMounted(async () => {
  try {
    actions.value = await backOff(async () => {
      const response = await axios.get('https://api.1.koishi.top/action/list');
      if (response.status != 200) {
        throw response;
      }
      return response.data;
    }, {
      numOfAttempts: 5,
    });
    console.log(JSON.parse(JSON.stringify(actions.value)));
  } catch (err) {
    console.error(err);
    failed.value = true;
  }
})
</script>