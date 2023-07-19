<template>
  <main class="font-splatoon2 bg-white dark:bg-gray-900 transition-colors">
    <nav class="fmt-border-color bg-gray-50 dark:bg-gray-800 mb-4 transition-colors">
      <div class="max-w-screen-xl flex flex-wrap items-center justify-between mx-auto p-4">
        <router-link to="/" class="flex items-center">
          <img src="/logo.svg" class="h-8 mr-3" alt="Flowbite Logo" />
          <span
            class="font-splatoon1 self-center text-2xl font-semibold whitespace-nowrap fmt-text-primary">SplatQuery</span>
        </router-link>
        <div class="flex items-center md:order-2 space-x-4">
          <button data-tooltip-target="tips-night-mode"
            class="inline-flex items-center justify-center fmt-text-secondary hover:bg-gray-100 w-10 h-10 dark:hover:bg-gray-700 focus:outline-none focus:ring-4 focus:ring-gray-200 dark:focus:ring-gray-700 rounded-lg text-sm p-2.5"
            @click="toggleTheme">
            <svg class="fill-current w-4 h-4" viewBox="0 0 18 20">
              <path v-if="theme != 'dark'"
                d="M17.8 13.75a1 1 0 0 0-.859-.5A7.488 7.488 0 0 1 10.52 2a1 1 0 0 0 0-.969A1.035 1.035 0 0 0 9.687.5h-.113a9.5 9.5 0 1 0 8.222 14.247 1 1 0 0 0 .004-.997Z">
              </path>
              <path v-if="theme == 'dark'"
                d="M10 15a5 5 0 1 0 0-10 5 5 0 0 0 0 10Zm0-11a1 1 0 0 0 1-1V1a1 1 0 0 0-2 0v2a1 1 0 0 0 1 1Zm0 12a1 1 0 0 0-1 1v2a1 1 0 1 0 2 0v-2a1 1 0 0 0-1-1ZM4.343 5.757a1 1 0 0 0 1.414-1.414L4.343 2.929a1 1 0 0 0-1.414 1.414l1.414 1.414Zm11.314 8.486a1 1 0 0 0-1.414 1.414l1.414 1.414a1 1 0 0 0 1.414-1.414l-1.414-1.414ZM4 10a1 1 0 0 0-1-1H1a1 1 0 0 0 0 2h2a1 1 0 0 0 1-1Zm15-1h-2a1 1 0 1 0 0 2h2a1 1 0 0 0 0-2ZM4.343 14.243l-1.414 1.414a1 1 0 1 0 1.414 1.414l1.414-1.414a1 1 0 0 0-1.414-1.414ZM14.95 6.05a1 1 0 0 0 .707-.293l1.414-1.414a1 1 0 1 0-1.414-1.414l-1.414 1.414a1 1 0 0 0 .707 1.707Z">
              </path>
            </svg>
          </button>
          <div id="tips-night-mode" role="tooltip" class="fmt-tips invisible opacity-0">
            {{ theme == 'dark' ? 'Light' : 'Dark' }} mode
            <div class="tooltip-arrow" data-popper-arrow></div>
          </div>
          <router-link to="/settings" type="button" data-tooltip-target="tips-user-icon"
            class="flex mr-3 text-sm bg-gray-800 rounded-full md:mr-0 focus:ring-4 focus:ring-gray-300 dark:focus:ring-gray-600">
            <span class="sr-only">Open user menu</span>
            <div class="relative w-10 h-10 overflow-hidden bg-gray-100 rounded-full dark:bg-gray-600">
              <img v-if="user" :src="user.picture" />
              <svg v-if="!user" class="absolute w-12 h-12 text-gray-400 -left-1" fill="currentColor" viewBox="0 0 20 20"
                xmlns="http://www.w3.org/2000/svg">
                <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd">
                </path>
              </svg>
            </div>
            <!-- <img class="w-8 h-8 rounded-full" src="/docs/images/people/profile-picture-3.jpg" alt="user photo"> -->
          </router-link>
          <div id="tips-user-icon" role="tooltip" class="fmt-tips invisible opacity-0">
            Settings
            <div class="tooltip-arrow" data-popper-arrow></div>
          </div>
        </div>
      </div>
    </nav>

    <div class="max-w-screen-xl mx-auto px-2 sm:px-4 md:px-6">
      <router-view />
    </div>

    <MessageQueue ref="mq" />
  </main>
</template>

<script setup>
import { onMounted, provide, ref } from 'vue'
import { initFlowbite } from 'flowbite'
import { useAuth } from '@websanova/vue-auth/src/v3.js';
import MessageQueue from './components/MessageQueue.vue';

onMounted(initFlowbite);

const auth = useAuth();

const mq = ref();
const theme = ref(localStorage.theme)
const user = ref();

provide('mq', mq);

const toggleTheme = () => {
  localStorage.theme = theme.value != 'dark' ? 'dark' : 'light';
  updateTheme();
}

const updateTheme = () => {
  if (localStorage.theme === 'dark' || (!('theme' in localStorage) && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
    theme.value = 'dark';
    document.documentElement.classList.add('dark')
  } else {
    theme.value = 'light';
    document.documentElement.classList.remove('dark')
  }
}

updateTheme();

onMounted(() => {
  const data = auth.remember();
  user.value = data && JSON.parse(data);
})
</script>

<style>
@import '@/assets/css/base.css';
</style>
