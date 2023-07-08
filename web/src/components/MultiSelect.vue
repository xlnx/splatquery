<template>
  <div class="multiselect" :class="msgInvalid ? 'invalid' : ''">
    <input :id="id" :value="content" :placeholder="placeholder" type="text"
      @change="$emit('update:value', $event.target.value.split(','))">
    <p class="mt-2 text-sm text-red-600 dark:text-red-500" v-if="msgInvalid">
      {{ msgInvalid }}
    </p>
  </div>
</template>

<script setup>
import TomSelect from 'tom-select';
import { computed, onMounted, ref, watch } from 'vue';

const props = defineProps({
  id: String,
  value: {
    type: Array,
    default: [],
  },
  options: {
    type: Array,
    default: [],
  },
  tagClass: {
    type: String,
    default: 'h-16 sm:h-24 aspect-[1/1]',
  },
  showTag: {
    type: Boolean,
    default: true,
  },
  placeholder: {
    type: String,
    default: '',
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  msgInvalid: {
    type: String,
    default: null,
  }
})

const content = computed(() => props.value.join(','))

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
    maxItems: 9999,
    // plugins: {
    //   remove_button: {
    //     title: 'Remove this item',
    //   },
    //   restore_on_backspace: {},
    //   change_listener: {},
    // },
    render: {
      option: ({ id, name, url }, escape) => {
        return `
          <div class="p-0">
            <img class="inline-block h-12" src="${url}">
            ${name}
          </div>`;
      },
      item: ({ id, name, url }, escape) => {
        // FIXME: add i18n
        const tag = !props.showTag ? '' : `
        <div class="
          absolute
          bg-gray-900
          text-white
          rounded
          bottom-0
          left-1/2
          -translate-x-1/2
          translate-y-0
          overflow-ellipsis
          overflow-hidden
          max-w-[95%]
          whitespace-nowrap
          px-1
          pointer-events-none
          text-xs xs:test-sm lg:text-md"
        >
          ${name}
        </div>
        `
        return `
        <div class="relative m-1">
          <div class="
            bg-gray-200
            dark:bg-gray-600
            ${props.tagClass}
            overflow-hidden 
            rounded-md 
            cursor-pointer
            hover:ring-4"
          >
            <img class="h-full" src="${url}" />
          </div>
          ${tag}
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
