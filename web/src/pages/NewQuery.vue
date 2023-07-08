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
        <button class="fmt-button" @click="create" :disabled="!!submission">
          <svg v-if="submission == 'create'"
            class="inline-block align-[-2px] animate-spin -ml-1 sm:-ml-2 mr-2 sm:mr-3 h-5 w-5 text-white"
            xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
            </path>
          </svg>
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

onMounted(initFlowbite);

const type = ref('pvp');
const form = ref();
const submission = ref();

const create = async () => {
  if (submission.value) {
    return;
  }
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
