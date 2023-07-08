<template>
  <div v-if="!failed">
    <Loading />
  </div>
  <div v-if="failed">
    <ServerDown />
  </div>
</template>

<script setup>
import { onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { initFlowbite } from 'flowbite'
import { useAuth } from '@websanova/vue-auth/src/v3.js';
import Loading from '../components/Loading.vue';

onMounted(initFlowbite);

const auth = useAuth();
const route = useRoute();
const failed = ref();

onMounted(async () => {
  try {
    auth.oauth2('google', {
      url: 'https://api.1.koishi.top/auth/google',
      code: true,
      data: {
        code: route.query.code,
        redirect_uri: new URL(route.path, window.location.origin).href,
      },
      state: route.query.state,
    })
  } catch (err) {
    failed.value = true;
  }
})

</script>
