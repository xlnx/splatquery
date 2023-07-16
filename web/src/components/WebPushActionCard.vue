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
      <div class="m-2" v-for="ua in agents" :key="ua.endpoint">
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
import { ref, onMounted, computed, inject } from 'vue';
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

const mq = inject('mq');
const submission = ref();
const endpoint = ref();
const active = ref(props.defaultActive);
const agents = ref(props.defaultConfig || []);
const isRegistered = computed(() => agents.value.some(e => e.endpoint && endpoint.value == e.endpoint));

const subscribeImpl = async () => {
  const { Notification } = window;
  if (!Notification) {
    throw 'Notification is not supported by your browser.';
  }
  try {
    await Notification.requestPermission();
  } catch (err) {
    throw 'Notification permission denied.'
  }
  const { serviceWorker } = navigator;
  if (!serviceWorker) {
    throw 'You have to quit private mode to use webpush, or maybe your browser doesn\'t support service workers.'
  }
  let registration = null;
  try {
    registration = await serviceWorker.register('/sw.js', { scope: '/', type: 'module' });
    if (!registration) { throw null; }
  } catch (err) {
    throw 'Register service worker failed, please contact the developer for help.'
  }
  const { pushManager } = registration;
  if (!pushManager) {
    throw 'WebPush is not supported by your browser.'
  }
  let subInfo = null;
  try {
    subInfo = await pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: "BDKNzkxVCQM1T131qz1Ctoz3f8t2sNge-uD7D216Wi1rrVaOYfl1r_ZYNKD2LgYAVWjXVZdUHvU0BNnVhdGJSA0",
    });
  } catch (err) {
    throw 'WebPush permission denied.'
  }
  const { endpoint, keys } = subInfo.toJSON()
  const info = new UAParser(navigator.userAgent).getResult()
  const agent = {
    endpoint, keys,
    browser: info.browser.name,
    os: `${info.os.name} ${info.os.version}`,
    // device: ua.device.name,
  }
  // submit agent to server
  let id = await backOff(async () => {
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
  agents.value.push({ id, ...agent });

  mq.value.success("Subscribe success.")
}

const dismissImpl = async ({ id }) => {
  // submit agent to server
  await backOff(async () => {
    const response = await axios.post(import.meta.env.VITE_API_SERVER + `/action/delete?id=${id}`)
    if (response.status != 200) {
      throw response;
    }
    return response.data;
  }, {
    numOfAttempts: 5,
  });
  // update ui
  const idx = agents.value.findIndex(e => e.id == id);
  if (idx >= 0) {
    agents.value.splice(idx, 1);
  }

  mq.value.info("Dismiss success.")
}

const subscribe = async () => {
  submission.value = 'subscribe';
  try {
    await subscribeImpl();
  } catch (err) {
    mq.value.error(err);
  }
  submission.value = null;
}

const dismiss = async (ua) => {
  submission.value = ua.endpoint;
  try {
    await dismissImpl(ua);
  } catch (err) {
    mq.value.error(err);
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
    if (active.value) {
      mq.value.success("WebPush notifications on.")
    } else {
      mq.value.info("WebPush notifications off.")
    }
  } catch (err) {
    mq.value.error(err);
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
