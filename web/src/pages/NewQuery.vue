<template>
  <div class="max-w-screen-md mx-auto pb-8 fmt-form">

    <!-- Breadcrumb -->
    <nav class="flex justify-between">
      <ol class="inline-flex items-center mb-3 sm:mb-0">
        <span class="fmt-breadcrumb-item">New Query</span>
        <span class="mx-2 text-gray-400">/</span>
        <select :value="type" class="fmt-form-input w-24" @change="type = $event.target.value">
          <option value="pvp" selected>PVP</option>
          <option value="coop">Coop</option>
          <option value="gears">Gears</option>
        </select>
      </ol>
    </nav>

    <div class="px-2 pt-0 sm:pt-4">
      <div class="space-y-4">
        <PVPQuery v-if="type == 'pvp'" ref="form" :disabled="!!submission" />
        <CoopQuery v-if="type == 'coop'" ref="form" :disabled="!!submission" />
        <div id="gears" class="dark:text-white" v-if="type == 'gears'">
          Not implemented yet
        </div>
      </div>
      <div class="w-full mt-8 flex justify-end">
        <button class="fmt-button fmt-lg" @click="create" :disabled="!!submission">
          <LoadingCircle class="h-5 w-5" v-if="submission == 'create'" />
          <span>Create!</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { initFlowbite } from 'flowbite'
import axios from 'axios';
import PVPQuery from '../components/PVPQuery.vue';
import CoopQuery from '../components/CoopQuery.vue';
import LoadingCircle from '../components/LoadingCircle.vue';

onMounted(initFlowbite);

const type = ref('pvp');
const form = ref();
const submission = ref();

const create = async () => {
  const query = form.value.validate();
  if (!query) {
    return;
  }
  submission.value = 'create';
  try {
    const form = { type: type.value, ...query };
    console.log(form);
    await axios.post('https://api.1.koishi.top/query/new', form);
    window.location.replace('/query/list');
  } catch (err) {
    console.error(err);
  }
  submission.value = null;
}
</script>
