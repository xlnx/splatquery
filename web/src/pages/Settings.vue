<template>
  <div class="max-w-screen-md mx-auto pb-8 fmt-form" v-if="!!form">

    <div class="px-2 pt-0 sm:pt-4">
      <div class="space-y-4">
        <!-- </div> -->
        <div>
          <label class="fmt-form-label">
            Language
          </label>
          <Select id="languages" v-model:value="form.language" :options="languages" :disabled="!!submission" />
        </div>
        <!-- <div class="grid grid-cols-2 gap-2 sm:gap-4"> -->
        <div>
          <label class="fmt-form-label">
            Time Zone
          </label>
          <Select id="regions" v-model:value="form.timeZone" :options="timeZones" :disabled="!!submission" />
        </div>
        <!--  -->
        <label class="fmt-form-label">
          Active Hours
        </label>

        <!-- <div class="sm:hidden">
          <label for="tabs" class="sr-only">Select your country</label>
          <select id="tabs"
            class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
            <option>Profile</option>
            <option>Canada</option>
            <option>France</option>
            <option>Germany</option>
          </select>
        </div> -->
        <!-- hidden sm: -->
        <ul data-tabs-toggle="#hrs-day-li" role="tablist"
          class="flex text-sm font-medium text-center text-gray-500 divide-x divide-gray-200 rounded-lg shadow dark:divide-gray-700 dark:text-gray-400">
          <li class="w-full" v-for="(title, day) in weekdays ">
            <button role="tab" :data-tabs-target="`#hrs-day-${day}`" class="fmt-tab-pill"
              :class="day == 0 ? 'rounded-l-lg' : day == 6 ? 'rounded-r-lg' : ''">
              {{ title }}
            </button>
          </li>
        </ul>

        <div id="hrs-day-li">
          <div :id="`hrs-day-${day}`" role="tabpanel" class="hidden h-fit" v-for=" day  in  new Array(7).keys() ">
            <div class="grid grid-cols-2 gap-2 sm:gap-4">
              <div class="flex flex-col"
                v-for="(hrs, i) in [dayHrsNever.concat(dayHrs.slice(0, 6)), dayHrsNever.concat(dayHrs.slice(6))]">
                <select v-model="form.dayHrs[day][i]" multiple class="fmt-form-input w-full flex-1">
                  <option :value="hr.id" v-for="hr in hrs">
                    {{ hr.name }}
                  </option>
                </select>
              </div>
            </div>
          </div>
        </div>

      </div>
      <div class="w-full mt-8 flex justify-end">
        <button class="fmt-button fmt-lg" @click="update" :disabled="!!submission">
          <LoadingCircle class="h-5 w-5" v-if="submission == 'update'" />
          <span>Update</span>
        </button>
      </div>
    </div>
  </div>

  <div v-if="!form && !failed">
    <Loading />
  </div>

  <div v-if="failed">
    <ServerDown />
  </div>
</template>

<script setup>
import { computed, onMounted, onUpdated, ref } from 'vue';
import { initFlowbite } from 'flowbite';
import axios from 'axios';
import { backOff } from 'exponential-backoff';
import Select from '../components/Select.vue'
import LoadingCircle from '../components/LoadingCircle.vue'
import Loading from '../components/Loading.vue';
import ServerDown from '../components/ServerDown.vue';

onMounted(initFlowbite)
onUpdated(initFlowbite)

const form = ref();
const submission = ref();
const failed = ref();

const toLocalDayHrs = (hrs, tz) => {
  const x = Math.floor((tzOffsets.jst - tzOffsets[tz]) / 2);
  hrs = hrs.flatMap((e, j) => {
    const li = [];
    for (let i = 0; i < [4, 3][j]; ++i) {
      li.push(e % (1 << 12));
      e /= (1 << 12);
    }
    return li;
  });
  hrs = hrs.map(e => e << x);
  hrs = hrs.map((e, i) =>
    (e | (hrs[(i + 7 - 1) % 7] >> 12) & ((1 << x) - 1))
  );
  return hrs.map(e => {
    const mask = e & ((1 << 12) - 1);
    let li = [[], []];
    for (let t = 0; t < 12; ++t) {
      if (mask & (1 << t)) {
        li[+(t >= 6)].push(t);
      }
    }
    return li;
  })
}

const toJstDayHrs = (hrs, tz) => {
  const x = Math.floor((tzOffsets.jst - tzOffsets[tz]) / 2);
  hrs = hrs.map(li => {
    li = li.flatMap(e => e.findIndex(e => e == -1) >= 0 ? [] : e)
    let v = 0;
    for (let e of li) {
      v |= 1 << e;
    }
    return v;
  });
  hrs = hrs.map((e, i) =>
    (e | (hrs[(i + 1) % 7] & ((1 << x) - 1)) << 12)
  );
  hrs = hrs.map(e => e >> x);
  return [hrs.slice(0, 4), hrs.slice(4)].map(li => {
    let v = 0;
    for (let e of li.reverse()) {
      v *= (1 << 12);
      v += e & ((1 << 12) - 1);
    }
    return v;
  })
}

onMounted(async () => {
  try {
    const data = await backOff(async () => {
      const response = await axios.get(import.meta.env.VITE_API_SERVER + '/user/list');
      if (response.status != 200) {
        throw response;
      }
      return response.data;
    }, {
      numOfAttempts: 5,
    });
    form.value = {
      language: data.language,
      timeZone: data.time_zone,
      dayHrs: toLocalDayHrs(data.day_hrs, data.time_zone),
    }
  } catch (err) {
    console.error(err);
    failed.value = true;
  }
});

const update = async () => {
  submission.value = 'update';
  try {
    const data = {
      language: form.value.language,
      time_zone: form.value.timeZone,
      day_hrs: toJstDayHrs(form.value.dayHrs, form.value.timeZone),
    }
    await axios.post(import.meta.env.VITE_API_SERVER + `/user/update`, data);
  } catch (err) {
    console.error(err);
  }
  submission.value = null;
}

const timeZones = [
  {
    id: 'jst',
    name: 'JST/Tokyo',
    url: `/img/region/jp.svg`,
  },
  {
    id: 'pt',
    name: 'PT/SF',
    url: `/img/region/na.svg`,
  },
  {
    id: 'cest',
    name: 'CEST/Berlin',
    url: `/img/region/eu.svg`,
  },
  {
    id: 'cst',
    name: 'CST/Beijing',
    url: `/img/region/cn.svg`,
  },
]

const languages = [
  {
    id: 'enus',
    name: 'English (US)',
    url: `/img/region/na.svg`
  },
]

const weekdays = [
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
  "Sunday",
]

const dayHrsNever = [
  {
    id: -1,
    name: 'Never',
  }
]

const tzOffsets = {
  jst: +9,
  pt: -7,
  cest: +2,
  cst: +8,
}

const dayHrs = computed(() => {
  const dt = Math.abs(tzOffsets[form.value.timeZone] % 2);
  return [...Array(12).keys()].map(i => (
    {
      id: i,
      name: `${2 * i + dt}:00 - ${2 * i + dt + 2}:00`,
    }
  ))
})
</script>
