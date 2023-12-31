<template>
  <Home v-if="showLoginBar"></Home>
  <ListQueries v-if="showMain"></ListQueries>
  <div v-if="!!pushDenyReason">
    <!-- TODO: impl style -->
    {{ `WebPush permisson denied: [${pushDenyReason}]` }}
  </div>

  <div v-if="!showLoginBar && !showMain && !pushDenyReason">
    <Loading />
  </div>
</template>

<script setup>
import { useAuth } from '@websanova/vue-auth/src/v3';
import { inject, onMounted, ref } from 'vue';
import axios from 'axios';
import UAParser from 'ua-parser-js';
import { tryGetSubInfo, getSubInfo } from '../webpush';
import { backOff } from 'exponential-backoff';
import Home from './Home.vue';
import Loading from '../components/Loading.vue';
import ListQueries from './ListQueries.vue';

const auth = useAuth();

const mq = inject('mq');
const showLoginBar = ref();
const showMain = ref();
const pushDenyReason = ref();

const subscribe = async ({ endpoint, keys }) => {
  const info = new UAParser(navigator.userAgent).getResult()
  const agent = {
    endpoint, keys,
    browser: info.browser.name,
    os: `${info.os.name} ${info.os.version}`,
    pwa: 1,
    // device: ua.device.name,
  }
  // submit ep to server
  await backOff(async () => {
    const response = await axios.post(import.meta.env.VITE_API_SERVER + '/action/webpush/subscribe', agent)
    if (response.status != 200) {
      throw response;
    }
    return response.data;
  }, {
    numOfAttempts: 5,
  });
}

const validate = async ({ endpoint, keys }) => {
  // TODO: test whether this endpoint is valid at server side
}

onMounted(async () => {
  // check auth status
  await auth.load();
  if (!auth.check()) {
    showLoginBar.value = true;
    return;
  }

  let subInfo = await tryGetSubInfo();
  if (localStorage.__pwa_ep) {
    if (!subInfo || subInfo && localStorage.__pwa_ep != subInfo.endpoint) {
      // endpoint invalidated
      localStorage.removeItem('__pwa_ep');
      subInfo = null;
    }
  }

  // ep valid
  if (localStorage.__pwa_ep) {
    showMain.value = true;
    validate(subInfo);
    return;
  }

  // check webpush permission
  let reason;
  if (!subInfo) {
    try {
      subInfo = await getSubInfo();
    } catch (err) {
      // TODO: impl reasons
      reason = 'permission';
    }
  }
  if (!subInfo) {
    // permission denied
    pushDenyReason.value = reason;
    return;
  }

  // permission ok
  localStorage.__pwa_ep = subInfo.endpoint;
  try {
    await subscribe(subInfo);
  } catch (err) {
    mq.value.error(err);
    return;
  }

  showMain.value = true;
})
</script>
