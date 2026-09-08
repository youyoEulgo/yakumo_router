<script setup lang="ts">
defineProps<{
  collapsed: boolean;
  title: string;
}>();

const emit = defineEmits<{
  toggle: [];
}>();
</script>

<template>
  <button
    type="button"
    class="collapsible-header list-header"
    :aria-expanded="!collapsed"
    @click="emit('toggle')"
  >
    <h2>{{ title }}</h2>
    <span class="collapse-button" :class="{ collapsed }" aria-hidden="true">▾</span>
  </button>

  <slot v-if="!collapsed"></slot>
</template>

<style scoped>
.list-header {
  margin-top: 6px;
}

.list-header:first-child {
  margin-top: 0;
}
</style>
