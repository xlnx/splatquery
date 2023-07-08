<template>
  <div>
    <label class="fmt-form-label">
      Stages
    </label>
    <MultiSelect id="coop_stages" v-model:value="form.stages" :options="coopStages" :disabled="disabled"
      tagClass="h-16 sm:h-24 aspect-[2/1]" placeholder="Pick a stage..."
      :msgInvalid="!ok && !stagesValid ? 'Pick at least one stage.' : null" />
  </div>
</template>

<script setup>
import { onMounted, ref, computed } from 'vue'
import { initFlowbite } from 'flowbite'
import MultiSelect from './MultiSelect.vue'
import { getCoopStageImgUrl } from '../utils'

onMounted(initFlowbite);

const props = defineProps({
  disabled: {
    type: Boolean,
    default: false,
  }
});

const formDefault = {
  stages: [],
}

const form = ref(formDefault);
const ok = ref(true);
const stagesValid = computed(() => form.value.stages.length > 0);
const formValid = computed(() => stagesValid.value);

const validate = () => {
  if (ok.value = formValid.value) {
    return form.value;
  }
}

defineExpose({
  validate
})

const coopStages = [
  {
    "name": "Gone Fission Hydroplant",
    "id": 7,
  },
  {
    "name": "Jammin' Salmon Junction",
    "id": 8,
  },
  {
    "name": "Spawning Grounds",
    "id": 1,
  },
  {
    "name": "Sockeye Station",
    "id": 2,
  },
  {
    "name": "Marooner's Bay",
    "id": 6,
  }
].map(({ id, name }) => ({
  id,
  name,
  url: getCoopStageImgUrl(id)
}));

</script>
