<template>
  <div class="max-w-screen-md mx-auto pb-8 fmt-form" v-if="query">

    <!-- Breadcrumb -->
    <nav class="flex justify-between px-2 sm:px-4 md:px-6">
      <ol class="inline-flex items-center mb-3 sm:mb-0">
        <span class="fmt-breadcrumb-item">Edit Query</span>
        <span class="mx-2 text-gray-400">/</span>
        <span class="fmt-breadcrumb-item">{{ query.type }}</span>
        <span class="mx-2 text-gray-400">/</span>
        <span class="fmt-breadcrumb-item">{{ qid }}</span>
      </ol>
    </nav>

    <div class="px-4 sm:px-6 md:px-8 pt-0 sm:pt-4">
      <div class="space-y-4">
        <PVPQuery v-if="query.type == 'pvp'" ref="form" :default="query" :disabled="!!submission" />
        <CoopQuery v-if="query.type == 'coop'" ref="form" :default="query" :disabled="!!submission" />
        <div id="gears" class="dark:text-white" v-if="query.type == 'gears'">
          Not implemented yet
        </div>
      </div>
      <div class="w-full mt-8 flex justify-end space-x-4">
        <button class="fmt-button fmt-lg" @click="update" :disabled="!!submission">
          <LoadingCircle class="h-5 w-5" v-if="submission == 'update'" />
          <span>Update</span>
        </button>
        <button class="fmt-button fmt-alert fmt-lg" @click="remove" :disabled="!!submission">
          <LoadingCircle class="h-5 w-5" v-if="submission == 'remove'" />
          <span>Delete</span>
        </button>
      </div>
    </div>
  </div>

  <div v-if="!query && !failed">
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
import PVPQuery from '../components/PVPQuery.vue';
import CoopQuery from '../components/CoopQuery.vue';
import LoadingCircle from '../components/LoadingCircle.vue';
import Loading from '../components/Loading.vue';
import ServerDown from '../components/ServerDown.vue';

onMounted(initFlowbite);

const props = defineProps({
  qtype: String,
  qid: Number,
})

const query = ref();
const form = ref();
const failed = ref();
const submission = ref();

onMounted(async () => {
  try {
    if (!props.qtype) {
      throw `qtype=[${props.qtype}] is invalid`;
    }
    if (!Number.isInteger(props.qid)) {
      throw `qid=[${props.qid}] is not an integer`;
    }
    const li = await backOff(async () => {
      const response = await axios.get(import.meta.env.VITE_API_SERVER + `/query/list?qtype=${props.qtype}&qid=${props.qid}`);
      if (response.status != 200) {
        throw response;
      }
      return response.data;
    }, {
      numOfAttempts: 5,
    });
    if (li.length != 1) {
      throw `li.length:[${li.length}] != [1]`;
    }
    const { qid, config } = li[0];
    if (qid != props.qid) {
      throw `qid:[${qid}]!=[${props.qid}]`;
    }
    query.value = config;
  } catch (err) {
    console.error(err);
    failed.value = true;
  }
})

const update = async () => {
  const query = form.value.validate();
  if (!query) {
    return;
  }
  submission.value = 'update';
  try {
    const data = { type: query.type.value, ...query };
    await axios.post(import.meta.env.VITE_API_SERVER + `/query/update?qid=${props.qid}`, data);
    window.location.replace('/query/list');
  } catch (err) {
    console.error(err);
  }
  submission.value = null;
}

const remove = async () => {
  submission.value = 'remove';
  try {
    await axios.post(import.meta.env.VITE_API_SERVER + `/query/delete?qid=${props.qid}&qtype=${props.qtype}`);
    window.location.replace('/query/list');
  } catch (err) {
    console.error(err);
  }
  submission.value = null;
}
</script>
