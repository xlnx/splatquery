<template>
  <div class="py-8 px-4 mx-auto max-w-screen-xl text-center lg:py-16">
    <h1
      class="font-splatoon1 mb-4 text-5xl font-extrabold tracking-tight leading-none text-gray-900 md:text-7xl lg:text-8xl dark:text-white">
      SplatQuery
    </h1>
    <p class="mb-8 text-lg font-normal text-gray-500 lg:text-xl sm:px-16 lg:px-48 dark:text-gray-400">
      Get notified about your favourite gear/schedule updates of splatoon 3.
    </p>
    <div class="flex flex-col space-y-4 sm:flex-row sm:justify-center sm:space-y-0 sm:space-x-4" v-if="showLoginBar">
      <button type="button"
        class="text-white bg-[#4285F4] hover:bg-[#4285F4]/90 focus:ring-4 focus:outline-none focus:ring-[#4285F4]/50 font-medium rounded-lg text-sm px-5 py-2.5 text-center inline-flex items-center dark:focus:ring-[#4285F4]/55 mr-2 mb-2"
        @click="login.google">
        <svg class="w-4 h-4 mr-2 -ml-1" focusable="false" data-prefix="fab" data-icon="google" role="img"
          xmlns="http://www.w3.org/2000/svg" viewBox="0 0 488 512">
          <path fill="currentColor"
            d="M488 261.8C488 403.3 391.1 504 248 504 110.8 504 0 393.2 0 256S110.8 8 248 8c66.8 0 123 24.5 166.3 64.9l-67.5 64.9C258.5 52.6 94.3 116.6 94.3 256c0 86.5 69.1 156.6 153.7 156.6 98.2 0 135-70.4 140.8-106.9H248v-85.3h236.1c2.3 12.7 3.9 24.9 3.9 41.4z">
          </path>
        </svg>
        Sign in with Google
      </button>
    </div>
    <div class="flex flex-col space-y-4 sm:flex-row sm:justify-center sm:space-y-0 sm:space-x-4" v-if="showMain">
      <a type="button" class="fmt-button" href="query/new">
        + New Query
      </a>
      <a type="button" class="fmt-button" href="query/list">
        Query List
      </a>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue';
import { initFlowbite } from 'flowbite'
import { useAuth } from '@websanova/vue-auth/src/v3.js';

onMounted(initFlowbite);

const auth = useAuth();

const login = {
  google: () => {
    // console.log(auth.token());
    auth.oauth2('google', {
      params: {
        client_id: '44914949790-b1csnno86fmhl0pnbv9jgfoc7tp8i97r.apps.googleusercontent.com',
        scope: 'https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile',
        access_type: 'offline',
        state: {
          staySignedIn: true,
        },
      },
    })
  }
}

const showLoginBar = ref(false);
const showMain = ref(false);

onMounted(async () => {
  await auth.load();
  if (auth.check()) {
    showMain.value = true;
  } else {
    showLoginBar.value = true;
  }
})

// const rule = ref("area");
// const toggleRule = (newRule) => rule.value = newRule;

// const toggleWebPush = async () => {
//   // FIXME: add alert when not available
//   if (!props.query.endpoints.webPush) {
//     const serviceWorker = window.navigator.serviceWorker;
//     const registration = await serviceWorker.getRegistration();
//     const subscription = await registration.pushManager.subscribe({
//       userVisibleOnly: true,
//       applicationServerKey: "BDKNzkxVCQM1T131qz1Ctoz3f8t2sNge-uD7D216Wi1rrVaOYfl1r_ZYNKD2LgYAVWjXVZdUHvU0BNnVhdGJSA0",
//     })
//     console.log(subscription);
//   }
//   props.query.endpoints.webPush = !props.query.endpoints.webPush;
// }
// const toggleIFTTT = () => props.query.endpoints.ifttt = !props.query.endpoints.ifttt;
</script>
