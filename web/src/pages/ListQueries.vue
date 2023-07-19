<template>
  <div class="grid grid-flow-row-dense grid-cols-1 sm:grid-cols-2 lg:grid-cols-3" v-if="queries">
    <div class="m-2 sm:m-3 lg:m-4">
      <router-link to="/query/new">
        <Card class="p-2">
          <div class="py-2 border-4 border-gray-200 dark:border-gray-600 border-dashed rounded-xl w-full h-full flex">
            <div class="m-auto font-bold text-2xl fmt-text-primary">
              <span>+ New Query</span>
            </div>
          </div>
        </Card>
      </router-link>
    </div>
    <div class="m-2 sm:m-3 lg:m-4" v-for="query in queries" :key="query.qid">
      <router-link :to="`/query/edit?qid=${query.qid}&qtype=${query.config.type}`">
        <QueryCard :qtype="query.config.type" :query="query.config" :createdTime="query.created_time" />
      </router-link>
    </div>
  </div>

  <div v-if="!queries && !failed">
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
import Card from '../components/Card.vue';
import QueryCard from '../components/QueryCard.vue';
import Loading from '../components/Loading.vue';
import ServerDown from '../components/ServerDown.vue';

onMounted(initFlowbite);

const mq = inject('mq');
const queries = ref();
const failed = ref();

onMounted(async () => {
  try {
    queries.value = await backOff(async () => {
      const response = await axios.get(import.meta.env.VITE_API_SERVER + '/query/list');
      if (response.status != 200) {
        throw response;
      }
      return response.data;
    }, {
      numOfAttempts: 5,
    });
  } catch (err) {
    mq.value.error(err);
    failed.value = true;
  }
})
</script>
