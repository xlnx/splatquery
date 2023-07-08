<template>
  <div>
    <label class="fmt-form-label">
      Modes
    </label>
    <MultiSelect id="pvp_modes" v-model:value="form.modes" :options="vsModes" :disabled="disabled"
      tagClass="h-16 sm:h-20 aspect-[1/1]" placeholder="Pick a mode..."
      :msgInvalid="!ok && !modesValid ? 'Pick at least one mode.' : null" />
  </div>
  <div>
    <label class="fmt-form-label">
      Rules
    </label>
    <MultiSelect id="pvp_rules" v-model:value="form.rules" :options="vsRules" :disabled="disabled"
      tagClass="h-16 sm:h-20 aspect-[1/1]" placeholder="Pick a rule..."
      :msgInvalid="!ok && !rulesValid ? 'Pick at least one rule.' : null" />
  </div>
  <div>
    <label class="fmt-form-label">
      Include Stages
    </label>
    <MultiSelect id="pvp_incl_stages" v-model:value="form.includes" :options="vsStages" :disabled="disabled"
      tagClass="h-16 sm:h-24 aspect-[2/1]" placeholder="Pick a stage..."
      :msgInvalid="!ok && !includesValid ? 'Pick at least one stage.' : null" />
  </div>
  <div>
    <label class="fmt-form-label">
      Exclude Stages
    </label>
    <MultiSelect id="pvp_excl_stages" v-model:value="form.excludes" :options="vsStages" :disabled="disabled"
      tagClass="h-16 sm:h-24 aspect-[2/1]" placeholder="Pick a stage..." />
  </div>
</template>

<script setup>
import { onMounted, ref, computed } from 'vue'
import { initFlowbite } from 'flowbite'
import MultiSelect from './MultiSelect.vue'
import { getModeImgUrl, getRuleImgUrl, getPVPStageImgUrl } from '../utils'

onMounted(initFlowbite);

const props = defineProps({
  disabled: {
    type: Boolean,
    default: false,
  },
  default: {
    type: Object,
    default: {
      modes: [],
      rules: [],
      includes: [],
      excludes: [],
    }
  }
});

const form = ref(props.default);
const ok = ref(true);
const modesValid = computed(() => form.value.modes.length > 0);
const rulesValid = computed(() => form.value.rules.length > 0);
const includesValid = computed(() => form.value.includes.length > 0);
const formValid = computed(() => modesValid.value && rulesValid.value && includesValid.value);

const validate = () => {
  if (ok.value = formValid.value) {
    const { includes, excludes } = form.value
    return { ...form.value, includes: includes.map((e) => parseInt(e)), excludes: excludes.map((e) => parseInt(e)) };
  }
}

defineExpose({
  validate
})

const vsModes = [
  {
    "name": "Challenge",
    "id": "challenge",
  },
  {
    "name": "Open",
    "id": "open",
  },
  {
    "name": "X",
    "id": "x",
  },
  {
    "name": "Event",
    "id": "event",
  }
].map(({ id, name }) => ({
  id, name,
  url: getModeImgUrl(id)
}))

const vsRules = [
  {
    "name": "Splat Zones",
    "id": "area"
  },
  {
    "name": "Tower Control",
    "id": "yagura"
  },
  {
    "name": "Rainmaker",
    "id": "hoko"
  },
  {
    "name": "Clam Blitz",
    "id": "asari"
  }
].map(({ id, name }) => ({
  id,
  name,
  url: getRuleImgUrl(id)
}));

const vsStages = [
  {
    "name": "Scorch Gorge",
    "id": 1,
  },
  {
    "name": "Eeltail Alley",
    "id": 2,
  },
  {
    "name": "Hagglefish Market",
    "id": 3,
  },
  {
    "name": "Undertow Spillway",
    "id": 4,
  },
  {
    "name": "Um'ami Ruins",
    "id": 5,
  },
  {
    "name": "Mincemeat Metalworks",
    "id": 6,
  },
  {
    "name": "Brinewater Springs",
    "id": 7,
  },
  {
    "name": "Barnacle & Dime",
    "id": 8,
  },
  {
    "name": "Flounder Heights",
    "id": 9,
  },
  {
    "name": "Hammerhead Bridge",
    "id": 10,
  },
  {
    "name": "Museum d'Alfonsino",
    "id": 11,
  },
  {
    "name": "Mahi-Mahi Resort",
    "id": 12,
  },
  {
    "name": "Inkblot Art Academy",
    "id": 13,
  },
  {
    "name": "Sturgeon Shipyard",
    "id": 14,
  },
  {
    "name": "MakoMart",
    "id": 15,
  },
  {
    "name": "Wahoo World",
    "id": 16,
  },
  {
    "name": "Humpback Pump Track",
    "id": 17,
  },
  {
    "name": "Manta Maria",
    "id": 18,
  }
].map(({ id, name }) => ({
  id,
  name,
  url: getPVPStageImgUrl(id)
}));

</script>
