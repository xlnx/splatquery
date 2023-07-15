<template>
  <div class="multiselect">
    <input :id="id" :value="value" :placeholder="placeholder" type="text"
      @change="$emit('update:value', $event.target.value)">
  </div>
</template>

<script setup>
import TomSelect from 'tom-select';
import { onMounted, ref, watch } from 'vue';

const props = defineProps({
  id: String,
  value: String,
  options: {
    type: Array,
    default: [],
  },
  placeholder: {
    type: String,
    default: '',
  },
  disabled: {
    type: Boolean,
    default: false,
  }
})

const select = ref();

watch(props, async (props, oldProps) => {
  if (select.value) {
    if (props.disabled) {
      select.value.disable()
    } else {
      select.value.enable()
    }
  }
})

const reload = () => {
  select.value = new TomSelect(`#${props.id}`, {
    create: false,
    valueField: 'id',
    labelField: 'name',
    searchField: 'name',
    options: props.options,
    maxItems: 1,
    render: {
      option: ({ id, name, url }, escape) => {
        return `
          <div class="px-2">
            <img class="inline-block h-6 sm:h-8" src="${url}">
            <span class="px-1">${name}</span>
          </div>`;
      },
      item: ({ id, name, url }) => {
        return `
        <div class="m-1">
          <span>
            <img class="inline-block h-6 sm:h-8" src="${url}" />
          </span>
          <span class="inline-block align-[-2px]">${name}</span>
        </div>
        `
      }
    }
  })
  if (props.disabled) {
    select.value.disable()
  }
}

onMounted(reload)

</script>
