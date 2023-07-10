<template>
  <ActionCard name="Web Push" brief="Receive notifications via your browser." id="webpush">
    <template v-slot:badge>
      <span class="flex flex-1 justify-between">
        <div class="pr-2">
          <LoadingCircle class="h-7 w-7" v-if="submission == 'toggle'" />
        </div>
        <div class="pr-4">
          <Toggle :checked="active" @toggle="toggle" :disabled="!!submission" />
        </div>
      </span>
    </template>
    <label class="fmt-form-label">
      Devices
    </label>
    <div class="grid grid-flow-row-dense grid-cols-1 sm:grid-cols-2 lg:grid-cols-1">
      <div class="m-2" v-for="ua in agents">
        <div class="flex fmt-webpush-ua w-full h-full rounded-xl border fmt-border-color">
          <div class="p-2 flex flex-1 space-x-2">
            <img class="inline-block w-12" :src="getBrowserImgUrl(ua.browser)" />
            <div class="flex-1">
              <div class="fmt-xs fmt-text-primary">{{ `${ua.browser} - ${ua.os}` }}</div>
              <label v-if="isRegistered && ua.endpoint == endpoint" class="fmt-badge-success">
                Current device
              </label>
            </div>
          </div>
          <div class="flex flex-col justify-end">
            <button class="inline-block w-full h-full fmt-button fmt-alert fmt-webpush-ua-dismiss fmt-sm"
              :disabled="!!submission" @click="dismiss(ua)">
              <LoadingCircle class="h-4 w-4" v-if="submission == ua.endpoint" />
              <span>Dismiss</span>
            </button>
          </div>
        </div>
      </div>
      <div class="m-2" v-if="!isRegistered">
        <Card class="flex fmt-webpush-ua">
          <button class="p-1 flex flex-1 space-x-2 justify-center" @click="subscribe" :disabled="!!submission">
            <div class="py-2 border-4 border-gray-200 dark:border-gray-600 border-dashed rounded-xl w-full h-full flex">
              <div class="m-auto text-lg fmt-text-secondary">
                <LoadingCircle class="h-4 w-4" v-if="submission == 'subscribe'" />
                <span>+ Current device</span>
              </div>
            </div>
          </button>
        </Card>
      </div>
    </div>
  </ActionCard>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue';
import { initFlowbite } from 'flowbite'
import axios from 'axios';
import UAParser from 'ua-parser-js'
import Card from '../components/Card.vue';
import ActionCard from '../components/ActionCard.vue';
import LoadingCircle from '../components/LoadingCircle.vue';
import Toggle from '../components/Toggle.vue';
import { getBrowserImgUrl } from '../utils';
import { backOff } from 'exponential-backoff';

onMounted(initFlowbite);

const props = defineProps({
  defaultActive: Boolean,
  defaultConfig: Array,
})

const submission = ref();
const endpoint = ref();
const active = ref(props.defaultActive);
const agents = ref(props.defaultConfig || []);
const isRegistered = computed(() => agents.value.some(e => e.endpoint && endpoint.value == e.endpoint));

const subscribeImpl = async () => {
  const reg = await navigator.serviceWorker.getRegistration();
  const sub = await reg.pushManager.subscribe({
    userVisibleOnly: true,
    applicationServerKey: "BDKNzkxVCQM1T131qz1Ctoz3f8t2sNge-uD7D216Wi1rrVaOYfl1r_ZYNKD2LgYAVWjXVZdUHvU0BNnVhdGJSA0",
  });
  const { endpoint, keys } = sub.toJSON()
  const info = new UAParser(navigator.userAgent).getResult()
  const agent = {
    endpoint, keys,
    browser: info.browser.name,
    os: `${info.os.name} ${info.os.version}`,
    // device: ua.device.name,
  }
  // submit agent to server
  await backOff(async () => {
    const response = await axios.post(import.meta.env.VITE_API_SERVER + '/action/webpush/subscribe', agent)
    if (response.status != 200) {
      throw response;
    }
    return response.data;
  }, {
    numOfAttempts: 5,
  });
  // update ui
  delete agent.p256dh;
  delete agent.auth;
  agents.value.push(agent);
}

const dismissImpl = async ({ endpoint }) => {
  // submit agent to server
  await backOff(async () => {
    const response = await axios.post(import.meta.env.VITE_API_SERVER + '/action/webpush/dismiss', { endpoint })
    if (response.status != 200) {
      throw response;
    }
    return response.data;
  }, {
    numOfAttempts: 5,
  });
  // update ui
  const idx = agents.value.findIndex(e => e.endpoint == endpoint);
  if (idx >= 0) {
    agents.value.splice(idx, 1);
  }
}

const subscribe = async () => {
  submission.value = 'subscribe';
  try {
    await subscribeImpl();
  } catch (err) {
    console.error(err);
  }
  submission.value = null;
}

const dismiss = async (ua) => {
  submission.value = ua.endpoint;
  try {
    await dismissImpl(ua);
  } catch (err) {
    console.error(err);
  }
  submission.value = null;
}

const toggle = async (newActive) => {
  submission.value = 'toggle';
  try {
    if (newActive && agents.value.length == 0) {
      // register current ua
      await subscribeImpl();
    }
    await backOff(async () => {
      const response = await axios.post(import.meta.env.VITE_API_SERVER + `/action/webpush/toggle?active=${!!newActive}`)
      if (response.status != 200) {
        throw response;
      }
      return response.data;
    }, {
      numOfAttempts: 5,
    });
    active.value = newActive;
  } catch (err) {
    console.error(err);
  }
  submission.value = null;
}

onMounted(async () => {
  const reg = await navigator.serviceWorker.getRegistration();
  const sub = await reg.pushManager.getSubscription();
  if (sub) {
    endpoint.value = sub.endpoint;
  }
})

</script>
