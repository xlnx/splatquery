<template>
  <div v-if="!failed">
    <Loading />
  </div>
  <div v-if="failed">
    <ServerDown />
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import { initFlowbite } from 'flowbite'
import { useAuth } from '@websanova/vue-auth/src/v3.js';
import Loading from '../components/Loading.vue';
import ServerDown from '../components/ServerDown.vue';

onMounted(initFlowbite);

const auth = useAuth();
const route = useRoute();
const failed = ref();

onMounted(async () => {
  try {
    const { data } = await auth.oauth2('google', {
      url: 'https://api.1.koishi.top/auth/google',
      code: true,
      data: {
        code: route.query.code,
        redirect_uri: new URL(route.path, window.location.origin).href,
      },
      state: route.query.state,
    })
    console.log(data);
    auth.remember(JSON.stringify(data));
  } catch (err) {
    failed.value = true;
  }
})

</script>
